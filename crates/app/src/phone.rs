//! The window's side of the phone's forms (D-28; EP-30a). A page's form reaches the window as a
//! [`PageRequest`] through the server's channel, drained in each pass, the window shown or
//! hidden (`Board::pass`), and is run on the window's own thread with the desk's own calls: an
//! owner task ticked as the Detail pane's box ticks it, an inbox message ticked with the
//! disposition the form carries, the owner's message written by the compose dialog's write on a
//! worker thread, a question or a permission request answered by the Decisions tile's rules.
//! What came of it goes back to the page in one line, and to the status bar and the rolling log
//! with the caller's identity, so the desk sees what the phone did. An order whose page stopped
//! waiting is dropped unrun; a screenshot run refuses every order, as it refuses the desk's.

use std::path::PathBuf;
use std::sync::mpsc::{self, Receiver, TryRecvError};
use std::thread;
use std::time::{Duration, Instant};

use crw_core::files;
use crw_core::inbox::mailman::{self, Sent};
use tokio::sync::oneshot;

use crate::board::Board;
use crate::decisions::{self, Draft};
use crate::server::{AnswerForm, Order, PageRequest, Said};

/// What a form for a request that no longer waits says (the brief's "already answered").
const ALREADY: &str = "Already answered: the session no longer waits on that request.";

/// How long a message sent from the phone is remembered, so that a second tap or a form sent
/// again from a page not yet reloaded writes it once (EP-30a's review).
const SENT_KEPT: Duration = Duration::from_secs(300);

/// The phone's messages being written, and those written lately (EP-30a).
#[derive(Default)]
pub struct Phone {
    sending: Vec<Sending>,
    /// By the project's lowercase name and the message's words: its id and when.
    sent: Vec<(String, String, String, Instant)>,
}

impl Phone {
    /// Whether a message from the phone is being written.
    pub fn is_sending(&self) -> bool {
        !self.sending.is_empty()
    }

    /// Why the same words to `project` are not written again: one being written, or one written
    /// within [`SENT_KEPT`].
    fn repeat(&mut self, project: &str, words: &str) -> Option<String> {
        self.sent.retain(|(.., at)| at.elapsed() < SENT_KEPT);
        let key = project.to_lowercase();
        if self
            .sending
            .iter()
            .any(|sending| sending.project.to_lowercase() == key && sending.words == words)
        {
            return Some(format!(
                "The same message to {project} is being written: it is not sent twice."
            ));
        }
        self.sent
            .iter()
            .find(|(name, said, ..)| *name == key && said == words)
            .map(|(_, _, id, _)| {
                format!(
                    "The same message went to {project} as {id} a moment ago: it is not sent \
                     twice. Change its words to send another."
                )
            })
    }
}

/// Where a message's write answers.
type Writing = Receiver<Result<Sent, String>>;

/// One message on its worker thread: whose, its words as written, who sent it, a session's words,
/// the write's answer and the page's reply.
struct Sending {
    project: String,
    words: String,
    caller: String,
    busy: Option<String>,
    answer: Writing,
    reply: oneshot::Sender<Said>,
}

impl Board {
    /// The pages' forms since the last pass, each run and answered, then the phone's messages
    /// that were written meanwhile (EP-30a).
    pub fn obey_pages(&mut self) {
        let orders = self.watch.take_orders();
        self.obey_orders(orders);
    }

    /// Runs `orders` and answers each; an order whose page stopped waiting, or let it go at its
    /// ten seconds, is dropped unrun, as a restart request's is (EP-37). A write reads the files
    /// again, and the pages are built afresh at once, so the page a form's redirect brings shows
    /// what it changed (the review's).
    pub(crate) fn obey_orders(&mut self, orders: Vec<PageRequest>) {
        let mut wrote = false;
        let mut ran = false;
        for request in orders {
            if request.reply.is_closed() || !request.hand.take() {
                crw_core::log::line(
                    "notice",
                    &format!(
                        "A form from {} was dropped: its page stopped waiting.",
                        request.caller
                    ),
                );
                continue;
            }
            ran = true;
            let PageRequest {
                order,
                caller,
                reply,
                ..
            } = request;
            if let Order::Send {
                project,
                text,
                urgent,
            } = order
            {
                match self.start_send(&project, &text, urgent) {
                    Ok((project, words, busy, answer)) => self.phone.sending.push(Sending {
                        project,
                        words,
                        caller,
                        busy,
                        answer,
                        reply,
                    }),
                    Err(why) => self.answer_page(reply, &caller, Err(why)),
                }
                continue;
            }
            let said = self.run_order(order, &mut wrote);
            self.answer_page(reply, &caller, said);
        }
        self.phone_pass();
        if wrote {
            self.loader.read_files();
        }
        if ran {
            self.pages.refresh();
        }
    }

    /// One order but a message: its outcome; `wrote` is set when a file was written.
    fn run_order(&mut self, order: Order, wrote: &mut bool) -> Said {
        let today = files::today();
        match order {
            Order::TickTask { project, id } => {
                let (name, folder) = self.writable_folder(&project)?;
                self.no_revert(&name)?;
                *wrote = true;
                self.tick_in(&folder, &name, &id, true, &today)
            }
            Order::TickMessage {
                project,
                id,
                disposition,
            } => {
                let (name, folder) = self.writable_folder(&project)?;
                self.no_revert(&name)?;
                *wrote = true;
                self.tick_message_in(&folder, &name, &id, &disposition, &today)
            }
            Order::Answer { key, request, form } => self.answer_from_page(key, &request, &form),
            Order::Send { .. } => Err("A message goes through its own write.".to_owned()),
        }
    }

    /// A revert open or running in `project` refuses the phone's writes there (the review's):
    /// at the desk its dialog is modal, and a write made under a running revert would be put
    /// back with the start's files and not redone.
    fn no_revert(&self, project: &str) -> Result<(), String> {
        if self.revert_open_in(project) {
            Err(format!(
                "A revert is open in {project}: tick or send there once it is done."
            ))
        } else {
            Ok(())
        }
    }

    /// Whether the revert's preview, or the revert it runs, is the project's (EP-15c).
    pub(crate) fn revert_open_in(&self, project: &str) -> bool {
        self.revert
            .as_ref()
            .is_some_and(|dialog| dialog.project.eq_ignore_ascii_case(project))
    }

    /// The page's answer to request `id` of the window's session `key`: sent while the session
    /// waits on it, or the deferred run resumed with it; "already answered" when it no longer
    /// waits. The response is built from the form by the tile's rules against the request as
    /// the window holds it.
    fn answer_from_page(&mut self, key: u64, id: &str, form: &AnswerForm) -> Said {
        if self.fixtures {
            return Err("A screenshot run answers nothing: its sessions are canned.".to_owned());
        }
        let Some(live) = self
            .sessions
            .live
            .iter()
            .find(|live| live.key == key && !live.is_chat())
        else {
            return Err(ALREADY.to_owned());
        };
        let title = live.title();
        if let Some(waiting) = live.waiting.iter().find(|w| w.request.id == id) {
            let request = waiting.request.clone();
            let draft = Draft::from_form(&request, &form.picks, &form.other, &form.reason);
            let response = draft
                .response(&request, form.verb)
                .map_err(|why| format!("{title}: not sent: {why}."))?;
            return self.answer_live(key, id, response);
        }
        let Some(open) = live
            .open
            .as_ref()
            .filter(|open| live.ended.is_some() && open.request.id == id)
        else {
            return Err(ALREADY.to_owned());
        };
        let request = open.request.clone();
        let draft = Draft::from_form(&request, &form.picks, &form.other, &form.reason);
        let response = draft
            .response(&request, form.verb)
            .map_err(|why| format!("{title}: not sent: {why}."))?;
        let words = decisions::answer_words(&request, &response);
        self.resume_deferred(key, response)
            .map(|()| format!("{title}: {words}; the session resumes with it."))
            .map_err(|why| why.words())
    }

    /// The compose dialog's refusals, then its write on a worker thread (EP-20c): a screenshot
    /// run's projects, a revert open there, a message of the flag alone; and the same words to
    /// the same project being written or written lately (the review's). The project's registry
    /// name, the words as written, a session's words there and where the write answers.
    fn start_send(
        &mut self,
        project: &str,
        text: &str,
        urgent: bool,
    ) -> Result<(String, String, Option<String>, Writing), String> {
        let (name, folder) = self.writable_folder(project)?;
        self.no_revert(&name)?;
        let busy = self.busy_projects().remove(&name.to_lowercase());
        if mailman::is_empty_message(&mailman::compose_text(text, false)) {
            return Err("The message is empty: nothing was sent.".to_owned());
        }
        let words = mailman::compose_text(text, urgent);
        if let Some(why) = self.phone.repeat(&name, &words) {
            return Err(why);
        }
        let (tx, answer) = mpsc::channel();
        let backups = self.store().backups.clone();
        let ledger = self.ledger().map(PathBuf::from);
        let ctx = self.ctx().clone();
        let (text, receiver) = (text.to_owned(), name.clone());
        let today = files::today();
        thread::Builder::new()
            .name("phone compose".to_owned())
            .spawn(move || {
                let sent = mailman::send_from_owner(
                    &folder,
                    &receiver,
                    &text,
                    urgent,
                    &backups,
                    &today,
                    ledger.as_deref(),
                );
                let _ = tx.send(sent);
                ctx.request_repaint();
            })
            .map_err(|e| format!("The message could not be sent: {e}"))?;
        Ok((name, words, busy, answer))
    }

    /// The phone's messages written since the last pass, each answered as the compose dialog
    /// says its own, the files read again.
    fn phone_pass(&mut self) {
        let mut done = Vec::new();
        for sending in std::mem::take(&mut self.phone.sending) {
            match sending.answer.try_recv() {
                Ok(answer) => done.push((sending, answer)),
                Err(TryRecvError::Empty) => self.phone.sending.push(sending),
                Err(TryRecvError::Disconnected) => done.push((
                    sending,
                    Err("the write's thread ended without a word".to_owned()),
                )),
            }
        }
        let mut wrote = false;
        for (sending, answer) in done {
            let said = match answer {
                Ok(sent) => {
                    wrote = true;
                    self.phone.sent.push((
                        sending.project.to_lowercase(),
                        sending.words.clone(),
                        sent.id.clone(),
                        Instant::now(),
                    ));
                    Ok(sent_words(&sending.project, &sent, sending.busy.as_deref()))
                }
                Err(why) => Err(format!("Not sent to {}: {why}", sending.project)),
            };
            self.answer_page(sending.reply, &sending.caller, said);
        }
        if wrote {
            self.loader.read_files();
        }
    }

    /// The outcome to the page, and to the status bar and the rolling log with the caller.
    fn answer_page(&mut self, reply: oneshot::Sender<Said>, caller: &str, said: Said) {
        match &said {
            Ok(words) => self.notify(format!("From the phone's page ({caller}): {words}")),
            Err(why) => self.warn(format!("From the phone's page ({caller}): {why}")),
        }
        let _ = reply.send(said);
    }
}

/// What a message written from the phone says, as the compose dialog says its own (EP-20c).
fn sent_words(project: &str, sent: &Sent, busy: Option<&str>) -> String {
    let kept = match &sent.copy {
        Some(copy) => format!("the file as it was is kept as {}", copy.display()),
        None => "roadmap/inbox.md created with its header".to_owned(),
    };
    let unlogged = match &sent.unlogged {
        Some(why) => format!(
            " {}; a revert would not redo it.",
            crate::compose::capital(why)
        ),
        None if !sent.revert_keeps => " It is longer than the ledger keeps: a revert of a \
                                       session there would name it instead of putting it back."
            .to_owned(),
        None => String::new(),
    };
    let session = match busy {
        Some(busy) => format!(
            " {}: it has read its inbox already, so the session after it triages the message.",
            crate::compose::capital(busy)
        ),
        None => String::new(),
    };
    format!("Sent {} to {project}; {kept}.{unlogged}{session}", sent.id)
}

#[cfg(test)]
mod tests {
    use std::collections::{BTreeMap, BTreeSet};
    use std::fs;
    use std::path::Path;
    use std::time::{Duration, Instant};

    use crw_core::ledger::Ledger;
    use crw_core::session::{Choice, Ended, Event, Launch, Outcome, Question, Request};
    use eframe::egui;
    use tokio::sync::oneshot::error::TryRecvError as Pending;

    use super::*;
    use crate::board::{Places, View};
    use crate::decisions::Verb;
    use crate::portfolio::tests::{Scratch, fake_store};
    use crate::sessions::Live;

    /// A board over the fake portfolio whose harbor lives in a scratch folder of its own, with
    /// two owner tasks and an inbox message, and a scratch ledger; never the tracked fixtures.
    fn board(scratch: &Scratch) -> (Board, PathBuf) {
        let places = Places {
            store: fake_store(scratch),
            ledger: Some(scratch.0.join("ledger.sqlite")),
            online: false,
            crash_log: None,
        };
        let ctx = egui::Context::default();
        let mut board = Board::new(places, &ctx, View::default(), egui::ThemePreference::Light);
        board.fixtures = false;
        board.portfolio = crate::portfolio::read_files(board.store());
        let folder = scratch.0.join("harbor-copy");
        fs::create_dir_all(folder.join("roadmap")).unwrap();
        fs::write(
            folder.join("roadmap").join("owner-tasks.md"),
            "# Owner tasks\n\n- [ ] OT-1 (EP-2, 2026-11-01) Look at the list.\n\
             - [ ] OT-2 (EP-3, 2026-11-02) Look at the pane.\n",
        )
        .unwrap();
        fs::write(
            folder.join("roadmap").join("inbox.md"),
            "# Inbox\n\n- [ ] IN-1 (from beta EP-4, 2026-11-02) The lock drifted.\n",
        )
        .unwrap();
        let row = board
            .portfolio
            .projects
            .iter_mut()
            .find(|row| row.name() == "harbor")
            .expect("the fake harbor");
        row.folder.as_mut().expect("its folder").path = folder.clone();
        (board, folder)
    }

    fn request(order: Order) -> (PageRequest, oneshot::Receiver<Said>) {
        let (reply, said) = oneshot::channel();
        let request = PageRequest {
            order,
            caller: "owner@example.com".to_owned(),
            reply,
            hand: crate::server::Hand::default(),
        };
        (request, said)
    }

    fn read(path: &Path) -> String {
        fs::read_to_string(path).unwrap()
    }

    #[test]
    fn the_phones_ticks_write_as_the_desks_and_say_so() {
        let scratch = Scratch::new("phone-ticks");
        let (mut board, folder) = board(&scratch);
        let tasks = folder.join("roadmap").join("owner-tasks.md");
        let inbox = folder.join("roadmap").join("inbox.md");
        // A task, the project named in another case.
        let (tick, mut said) = request(Order::TickTask {
            project: "HARBOR".to_owned(),
            id: "OT-1".to_owned(),
        });
        board.obey_orders(vec![tick]);
        let words = said.try_recv().unwrap().unwrap();
        assert!(words.starts_with("Ticked OT-1 in harbor;"), "{words}");
        assert!(read(&tasks).contains("- [x] OT-1 "), "{}", read(&tasks));
        assert!(read(&tasks).contains("- [ ] OT-2 "));
        assert!(
            board.notice_text().is_some_and(|notice| notice
                .starts_with("From the phone's page (owner@example.com): Ticked OT-1")),
            "{:?}",
            board.notice_text()
        );
        // A message without its disposition: refused, the file as it was.
        let before = read(&inbox);
        let (empty, mut said) = request(Order::TickMessage {
            project: "harbor".to_owned(),
            id: "IN-1".to_owned(),
            disposition: String::new(),
        });
        board.obey_orders(vec![empty]);
        let why = said.try_recv().unwrap().unwrap_err();
        assert!(why.starts_with("harbor IN-1: "), "{why}");
        assert_eq!(read(&inbox), before);
        // With it: ticked, the disposition on its line.
        let (message, mut said) = request(Order::TickMessage {
            project: "harbor".to_owned(),
            id: "IN-1".to_owned(),
            disposition: "parked in EP-9".to_owned(),
        });
        board.obey_orders(vec![message]);
        let words = said.try_recv().unwrap().unwrap();
        assert!(
            words.starts_with("Ticked IN-1 in harbor with its disposition"),
            "{words}"
        );
        assert!(
            read(&inbox).contains("- [x] IN-1 ") && read(&inbox).contains("parked in EP-9"),
            "{}",
            read(&inbox)
        );
        // Each tick is in the ledger, for a revert to redo.
        let ledger = Ledger::open(&scratch.0.join("ledger.sqlite")).unwrap();
        let ticks = ledger.events(crw_core::revert::TICK_KIND, None).unwrap();
        assert_eq!(ticks.len(), 2, "{ticks:?}");
        assert!(
            ticks
                .iter()
                .all(|row| row.project.as_deref() == Some("harbor"))
        );
        // An order whose page stopped waiting is dropped unrun, and so is one its page let go
        // at its ten seconds; a page cannot let go of an order the window took.
        let (dropped, said) = request(Order::TickTask {
            project: "harbor".to_owned(),
            id: "OT-2".to_owned(),
        });
        drop(said);
        board.obey_orders(vec![dropped]);
        assert!(read(&tasks).contains("- [ ] OT-2 "), "dropped unrun");
        let (let_go, _said) = request(Order::TickTask {
            project: "harbor".to_owned(),
            id: "OT-2".to_owned(),
        });
        assert!(let_go.hand.let_go());
        board.obey_orders(vec![let_go]);
        assert!(read(&tasks).contains("- [ ] OT-2 "), "let go, never run");
        let taken = crate::server::Hand::default();
        assert!(taken.take());
        assert!(
            !taken.let_go(),
            "taken first: the page says it is under way"
        );
        // A project the registry does not know.
        let (unknown, mut said) = request(Order::TickTask {
            project: "nowhere".to_owned(),
            id: "OT-1".to_owned(),
        });
        board.obey_orders(vec![unknown]);
        assert_eq!(
            said.try_recv().unwrap().unwrap_err(),
            "nowhere is not a registered project."
        );
        // A screenshot run refuses every order, its files untouched.
        board.fixtures = true;
        let (fixture, mut said) = request(Order::TickTask {
            project: "harbor".to_owned(),
            id: "OT-2".to_owned(),
        });
        let (sending, mut sent) = request(Order::Send {
            project: "harbor".to_owned(),
            text: "Again.".to_owned(),
            urgent: false,
        });
        board.obey_orders(vec![fixture, sending]);
        assert!(
            said.try_recv()
                .unwrap()
                .unwrap_err()
                .starts_with("A screenshot run writes nothing")
        );
        assert!(
            sent.try_recv()
                .unwrap()
                .unwrap_err()
                .starts_with("A screenshot run writes nothing")
        );
        assert!(read(&tasks).contains("- [ ] OT-2 "));
    }

    #[test]
    fn the_phones_message_is_written_by_the_composes_write_and_answered_once_written() {
        let scratch = Scratch::new("phone-send");
        let (mut board, folder) = board(&scratch);
        let inbox = folder.join("roadmap").join("inbox.md");
        // The flag alone is no message: refused at once.
        let (empty, mut said) = request(Order::Send {
            project: "harbor".to_owned(),
            text: " [urgent] ".to_owned(),
            urgent: true,
        });
        board.obey_orders(vec![empty]);
        assert_eq!(
            said.try_recv().unwrap().unwrap_err(),
            "The message is empty: nothing was sent."
        );
        // A message: written on its thread, answered once written.
        let (send, mut said) = request(Order::Send {
            project: "harbor".to_owned(),
            text: "Note the  trap.".to_owned(),
            urgent: true,
        });
        board.obey_orders(vec![send]);
        assert!(board.phone.is_sending());
        let started = Instant::now();
        let answer = loop {
            match said.try_recv() {
                Ok(answer) => break answer,
                Err(Pending::Empty) if started.elapsed() < Duration::from_secs(20) => {
                    std::thread::sleep(Duration::from_millis(20));
                    board.obey_orders(Vec::new());
                }
                Err(e) => panic!("no answer: {e:?}"),
            }
        };
        let words = answer.unwrap();
        assert!(
            words.starts_with("Sent IN-2 to harbor; the file as it was is kept as "),
            "{words}"
        );
        assert!(!board.phone.is_sending());
        assert!(
            read(&inbox).contains("- [ ] IN-2 (from the owner, ")
                && read(&inbox).ends_with(") Note the trap. [urgent]\n"),
            "{}",
            read(&inbox)
        );
        // The same words again, a second tap or a page not reloaded: written once (the
        // review's); other words go.
        let written = read(&inbox);
        let (again, mut said) = request(Order::Send {
            project: "harbor".to_owned(),
            text: "Note the trap.".to_owned(),
            urgent: true,
        });
        board.obey_orders(vec![again]);
        let why = said.try_recv().unwrap().unwrap_err();
        assert!(
            why.starts_with("The same message went to harbor as IN-2"),
            "{why}"
        );
        assert_eq!(read(&inbox), written);
        // Two taps at once: the second meets the first on its way.
        let (first, mut first_said) = request(Order::Send {
            project: "harbor".to_owned(),
            text: "Another note.".to_owned(),
            urgent: false,
        });
        let (second, mut second_said) = request(Order::Send {
            project: "harbor".to_owned(),
            text: "Another  note.".to_owned(),
            urgent: false,
        });
        board.obey_orders(vec![first, second]);
        let why = second_said.try_recv().unwrap().unwrap_err();
        assert!(why.contains("is being written"), "{why}");
        let started = Instant::now();
        loop {
            match first_said.try_recv() {
                Ok(answer) => {
                    let words = answer.unwrap();
                    assert!(words.starts_with("Sent IN-3 to harbor"), "{words}");
                    break;
                }
                Err(Pending::Empty) if started.elapsed() < Duration::from_secs(20) => {
                    std::thread::sleep(Duration::from_millis(20));
                    board.obey_orders(Vec::new());
                }
                Err(e) => panic!("no answer: {e:?}"),
            }
        }
        assert_eq!(read(&inbox).matches("Another note.").count(), 1);
    }

    fn question(text: &str, several: bool) -> Question {
        Question {
            text: text.to_owned(),
            header: text.to_owned(),
            options: ["Top", "Bottom", "Middle"]
                .iter()
                .map(|label| Choice {
                    label: (*label).to_owned(),
                    description: None,
                })
                .collect(),
            multi_select: several,
        }
    }

    fn asked(id: &str) -> Request {
        Request {
            id: id.to_owned(),
            tool: "AskUserQuestion".to_owned(),
            summary: "Which shelf?".to_owned(),
            reason: None,
            questions: vec![question("Shelf", false), question("Rows", true)],
            input: String::new(),
        }
    }

    fn form(verb: Verb, picks: &[(usize, &[usize])], reason: &str) -> AnswerForm {
        AnswerForm {
            verb,
            picks: picks
                .iter()
                .map(|(q, options)| (*q, options.iter().copied().collect::<BTreeSet<_>>()))
                .collect(),
            other: BTreeMap::new(),
            reason: reason.to_owned(),
        }
    }

    fn live(key: u64, project: &str, events: Vec<Event>) -> Live {
        let mut launch = Launch::new(project, Path::new(project), "fable", "go");
        launch.ep = Some("EP-4".to_owned());
        Live::canned(key, launch, "high", events)
    }

    #[test]
    fn an_answer_reaches_the_request_it_names_by_the_tiles_rules() {
        let scratch = Scratch::new("phone-answers");
        let (mut board, _) = board(&scratch);
        let deferred = vec![
            Event::Request(asked("req-9")),
            Event::TimedOut("req-9".to_owned()),
            Event::Exited(Ended {
                outcome: Outcome::Deferred,
                exit_code: Some(0),
                result: None,
                stderr: Vec::new(),
            }),
        ];
        board.sessions.live.extend([
            live(1, "harbor", vec![Event::Request(asked("req-1"))]),
            live(2, "meadow", deferred),
        ]);
        let answer = |board: &mut Board, key: u64, id: &str, form: AnswerForm| {
            let (order, mut said) = request(Order::Answer {
                key,
                request: id.to_owned(),
                form,
            });
            board.obey_orders(vec![order]);
            said.try_recv().unwrap()
        };
        // No answer given, a decline without its reason, a button of the other kind: refused
        // by the tile's rules before anything is sent.
        let why = answer(&mut board, 1, "req-1", form(Verb::Answer, &[], "")).unwrap_err();
        assert!(why.contains("no answer was given"), "{why}");
        let why = answer(&mut board, 1, "req-1", form(Verb::Decline, &[], " ")).unwrap_err();
        assert!(why.contains("a decline needs its reason"), "{why}");
        let why = answer(&mut board, 1, "req-1", form(Verb::Allow, &[], "")).unwrap_err();
        assert!(why.contains("it is a question"), "{why}");
        // A valid answer goes to the session's own call: this canned one has no process, so
        // the call says it has ended, which shows the answer was routed there.
        let why = answer(
            &mut board,
            1,
            "req-1",
            form(Verb::Answer, &[(0, &[1]), (1, &[0, 2])], ""),
        )
        .unwrap_err();
        assert!(
            why.contains("the answer was not sent: the session has ended"),
            "{why}"
        );
        // A request no longer waiting, a key the window does not hold, a deferred run's
        // request of another id: already answered.
        for (key, id) in [(1, "req-0"), (8, "req-1"), (2, "req-1")] {
            let why =
                answer(&mut board, key, id, form(Verb::Answer, &[(0, &[0])], "")).unwrap_err();
            assert_eq!(why, ALREADY, "{key} {id}");
        }
        // A deferred run's own request, refused by the rules before any resume.
        let why = answer(&mut board, 2, "req-9", form(Verb::Answer, &[], "")).unwrap_err();
        assert!(why.contains("no answer was given"), "{why}");
        // A chat's question is the Chat pane's: never answered from the page.
        let mut chat_launch = Launch::new("chat", Path::new("chat"), "fable", "go");
        chat_launch.conversation = Some("What is next?".to_owned());
        board.sessions.live.push(Live::canned(
            4,
            chat_launch,
            "high",
            vec![Event::Request(asked("req-6"))],
        ));
        let why = answer(&mut board, 4, "req-6", form(Verb::Answer, &[(0, &[0])], "")).unwrap_err();
        assert_eq!(why, ALREADY);
        // A screenshot run answers nothing.
        board.fixtures = true;
        let why = answer(&mut board, 1, "req-1", form(Verb::Answer, &[(0, &[0])], "")).unwrap_err();
        assert!(why.starts_with("A screenshot run answers nothing"), "{why}");
    }
}
