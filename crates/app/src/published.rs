//! What the window publishes for the phone's pages (D-14, D-28; EP-30): the project list as plain
//! data, read by the server's handlers on their own thread, since nothing the window holds can be
//! read there. The window builds it in each pass, shown or hidden (`Dashboard::logic`), at most
//! once a second and at once when a page asks for a fresh one, and sends it through a `watch`
//! channel when its content changed or a page asked; the version rises only with the content,
//! which is the change EP-30b's event stream names. A build that finds the same content renews
//! the snapshot's time without a word to the receivers. A page asks when the snapshot it holds is
//! older than [`FRESH_FOR`], and waits at most [`WAIT`] for the window's answer: a window too
//! busy to answer leaves it the older snapshot, whose time the page shows.

use std::collections::BTreeSet;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

use chrono::{DateTime, Utc};
use crw_core::badges::hosting::Class;
use crw_core::badges::stack::Kind;
use crw_core::registry::State;
use crw_core::roadmap::table::Counts;
use crw_core::session::Request;
use crw_core::tasks::OwnerItem;
use tokio::sync::watch;

use crate::badges::{self, Tone};
use crate::board::Board;
use crate::portfolio::{Filter, GroupKind, Member, ProjectRow, SortKey};

/// The window builds the snapshot at most this often, unless a page asks.
const EVERY: Duration = Duration::from_secs(1);

/// A page asks the window for a fresh snapshot when the one it holds is older than this.
pub const FRESH_FOR: Duration = Duration::from_secs(2);

/// How long a page waits for the window's fresh snapshot.
pub const WAIT: Duration = Duration::from_secs(2);

static NO_STACKS: BTreeSet<Kind> = BTreeSet::new();
static NO_CLASSES: BTreeSet<Class> = BTreeSet::new();

/// One snapshot: when the window built it, its version (raised only when the content changes)
/// and the content: the list, each project's page and the decisions (EP-30a).
#[derive(Debug, Clone, PartialEq)]
pub struct Published {
    pub at: DateTime<Utc>,
    pub version: u64,
    pub list: List,
    pub projects: Vec<Project>,
    pub decisions: Decisions,
}

impl Published {
    /// What the server holds before the window's first pass: nothing read yet.
    fn empty() -> Self {
        Self {
            at: DateTime::<Utc>::MIN_UTC,
            version: 0,
            list: List {
                reading: true,
                ..List::default()
            },
            projects: Vec::new(),
            decisions: Decisions::default(),
        }
    }

    /// The page of the project called `name`, its case aside.
    pub fn project(&self, name: &str) -> Option<&Project> {
        self.projects
            .iter()
            .find(|project| project.name.eq_ignore_ascii_case(name))
    }
}

/// What one build of the snapshot holds: the content without its time and version.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Parts {
    pub list: List,
    pub projects: Vec<Project>,
    pub decisions: Decisions,
}

impl Parts {
    /// Everything the pages read, as the window's board holds it at `now`.
    pub fn of(board: &Board, now: DateTime<Utc>) -> Self {
        let busy = board.busy_projects();
        let projects = board
            .portfolio
            .projects
            .iter()
            .map(|row| {
                let live = board.session_summary(row.name());
                let busy = busy.get(&row.name().to_lowercase()).cloned();
                let mut page = Project::of(row, live.as_deref(), busy);
                page.revert = board.revert_open_in(row.name());
                page
            })
            .collect();
        Self {
            list: List::of(board, now),
            projects,
            decisions: Decisions::of(board),
        }
    }
}

/// A project's page (EP-30a): what the Detail pane shows of it, as plain data.
#[derive(Debug, Clone, PartialEq)]
pub struct Project {
    pub name: String,
    pub state: State,
    /// The folder is there, so the tool may write into it.
    pub folder: bool,
    pub next: Next,
    /// The next brief's note as written, its own `**EP-n.**` label left out.
    pub note: Option<String>,
    /// What waits on the owner (D-10): the open items, and how many are resolved.
    pub waiting: Vec<Item>,
    pub resolved: usize,
    /// The older forms of a project that keeps `owner-tasks.md`, open, counted nowhere.
    pub uncounted: Vec<Item>,
    /// The inbox; none when the project keeps no `roadmap/inbox.md`.
    pub inbox: Option<Mail>,
    /// What says a session is in progress, as the list's line says it.
    pub sessions: Vec<String>,
    /// A session at work there or a revert open, in the compose dialog's words.
    pub busy: Option<String>,
    /// A revert is open or running there: the page offers no tick and no message until it is
    /// done (the review's).
    pub revert: bool,
}

/// One open item waiting on the owner.
#[derive(Debug, Clone, PartialEq)]
pub struct Item {
    pub id: Option<String>,
    pub gate: Option<String>,
    /// As written, without the gate its chip says.
    pub text: String,
    /// Its form, file and line, state and origin.
    pub meta: String,
    /// A line of `owner-tasks.md`, which the tool ticks; the older forms are read-only.
    pub tickable: bool,
}

/// A project's inbox: its open messages, and how many it holds.
#[derive(Debug, Clone, PartialEq)]
pub struct Mail {
    pub open: Vec<Message>,
    pub total: usize,
}

/// One open message.
#[derive(Debug, Clone, PartialEq)]
pub struct Message {
    pub id: String,
    /// `from <sender>, <date>` as far as the line says.
    pub from: String,
    pub urgent: bool,
    /// Its words, without a trailing `[urgent]` the chip says.
    pub text: String,
}

impl Project {
    /// `row`'s page, with the window's sessions' words `live` and a session's words `busy`.
    pub fn of(row: &ProjectRow, live: Option<&str>, busy: Option<String>) -> Self {
        let folder = row.folder.as_ref().map(|folder| folder.path.clone());
        let base = folder.clone().unwrap_or_default();
        let note = row.snapshot.current().and_then(|roadmap| {
            let next = roadmap.next()?;
            let note = roadmap.note(next.id)?;
            let label = format!("**{}.**", next.id);
            let text = note.text.strip_prefix(&label).unwrap_or(&note.text);
            Some(text.trim().to_owned())
        });
        let item = |item: &OwnerItem| Item {
            id: item.id.clone(),
            gate: item.gate.as_ref().map(ToString::to_string),
            text: crate::detail::without_gate(&item.text, item.gate.as_ref()).to_owned(),
            meta: crate::detail::meta(item, &base),
            tickable: item.is_tickable(),
        };
        let waiting = row.snapshot.waiting();
        let open: Vec<Item> = waiting
            .iter()
            .filter(|entry| entry.is_open())
            .map(item)
            .collect();
        let resolved = waiting.len() - open.len();
        let uncounted = row
            .snapshot
            .uncounted()
            .iter()
            .filter(|entry| entry.is_open())
            .map(item)
            .collect();
        let inbox = row.snapshot.inbox.as_ref().map(|inbox| Mail {
            open: inbox
                .open()
                .map(|message| Message {
                    id: message.id.clone(),
                    from: [message.from.clone(), message.date.clone()]
                        .into_iter()
                        .flatten()
                        .collect::<Vec<_>>()
                        .join(", "),
                    urgent: message.urgent,
                    text: if message.urgent {
                        crw_core::inbox::without_urgent(&message.text).to_owned()
                    } else {
                        message.text.clone()
                    },
                })
                .collect(),
            total: inbox.messages.len(),
        });
        Self {
            name: row.name().to_owned(),
            state: row.project.state,
            folder: folder.is_some(),
            next: Next::of(row),
            note,
            waiting: open,
            resolved,
            uncounted,
            inbox,
            sessions: crate::list::session_words(row, live),
            busy,
            revert: false,
        }
    }
}

/// The decisions page (EP-30a): what the Decisions tile holds, as plain data.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Decisions {
    /// The requests the window's sessions wait on, a chat's left to the Chat pane.
    pub waiting: Vec<Asking>,
    /// The ended runs deferred with a request open, which an answer resumes.
    pub deferred: Vec<Asking>,
    /// The sessions Claude Code reports waiting outside the window, never answered here.
    pub elsewhere: Vec<Elsewhere>,
}

impl Decisions {
    /// What waits for the owner's answer, counted as the tile's title counts it.
    pub fn count(&self) -> usize {
        self.waiting.len() + self.deferred.len() + self.elsewhere.len()
    }

    /// The board's sessions as the Decisions tile reads them.
    pub fn of(board: &Board) -> Self {
        let mut decisions = Decisions::default();
        for live in board.sessions.live.iter().filter(|live| !live.is_chat()) {
            let limit = live.launch_timeout().map(|limit| limit.as_secs() / 60);
            for waiting in &live.waiting {
                decisions.waiting.push(Asking {
                    key: live.key,
                    title: live.title(),
                    project: live.project.clone(),
                    request: waiting.request.clone(),
                    at: waiting.at,
                    how: How::Live { limit },
                });
            }
            if let Some(open) = live.open.as_ref().filter(|_| live.ended.is_some()) {
                decisions.deferred.push(Asking {
                    key: live.key,
                    title: live.title(),
                    project: live.project.clone(),
                    request: open.request.clone(),
                    at: open.at.with_timezone(&Utc),
                    how: How::Deferred {
                        hook: open.deferred.is_some(),
                    },
                });
            }
        }
        decisions.elsewhere = board
            .waiting_elsewhere()
            .into_iter()
            .map(|seen| {
                let (place, _) = crate::decisions::place(seen);
                Elsewhere {
                    project: seen.project.clone().unwrap_or_default(),
                    mode: seen.mode.as_str(),
                    words: seen.waits_for_you(place),
                    name: seen.name.clone(),
                }
            })
            .collect();
        decisions
    }
}

/// A request waiting for the owner's answer.
#[derive(Debug, Clone, PartialEq)]
pub struct Asking {
    /// The window's session, by its key.
    pub key: u64,
    pub title: String,
    pub project: String,
    pub request: Request,
    /// When it came, or when the run was deferred.
    pub at: DateTime<Utc>,
    pub how: How,
}

/// How a request waits.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum How {
    /// A running session's, with the project's time limit in minutes.
    Live { limit: Option<u64> },
    /// An ended run's, deferred by the defer route's hook or else at the time limit.
    Deferred { hook: bool },
}

/// A session waiting outside the window.
#[derive(Debug, Clone, PartialEq)]
pub struct Elsewhere {
    pub project: String,
    pub mode: &'static str,
    /// It "waits for you in VS Code", in the tile's words.
    pub words: String,
    pub name: Option<String>,
}

/// The project list as the window's list shows it (D-25): its state groups in the list's order
/// and sort, no filter; the folders not adopted, by name.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct List {
    /// The files are being read and no project is listed yet.
    pub reading: bool,
    /// Why the registry or the data root did not read.
    pub problem: Option<String>,
    pub groups: Vec<Group>,
    pub unadopted: Vec<String>,
}

/// A state's group.
#[derive(Debug, Clone, PartialEq)]
pub struct Group {
    pub state: State,
    pub projects: Vec<Line>,
}

impl Group {
    /// The group's heading, as the window's list names it.
    pub fn heading(&self) -> &'static str {
        crate::list::heading(GroupKind::State(self.state))
    }
}

/// One project as its row says it.
#[derive(Debug, Clone, PartialEq)]
pub struct Line {
    pub name: String,
    pub state: State,
    pub pinned: bool,
    /// A state change is proposed and not dismissed (D-6).
    pub proposes: bool,
    /// The project's autopilot is on (D-22).
    pub autopilot: bool,
    pub next: Next,
    pub counts: Counts,
    /// The owner's open items counted as waiting (D-10).
    pub owner_items: usize,
    /// Open inbox messages, and how many are urgent (D-13).
    pub messages: usize,
    pub urgent: usize,
    /// What says a session is in progress: the sessions' own words, then the files' evidence.
    pub sessions: Vec<String>,
    /// The week's share in tenths of a point (EP-18b), for a sort and a test.
    pub usage_tenths: Option<i64>,
    pub badges: Vec<Badge>,
}

/// The next brief, or why there is none.
#[derive(Debug, Clone, PartialEq)]
pub enum Next {
    Brief {
        id: String,
        size: Option<String>,
        /// The row's one-liner as written, its markup kept for the page to draw.
        title: String,
    },
    /// No next brief, in the row's words; `missing` when the folder is gone.
    None { words: String, missing: bool },
}

impl Next {
    /// `row`'s next brief, or why there is none in the row's words.
    fn of(row: &ProjectRow) -> Self {
        match (&row.folder, row.next()) {
            (None, _) => Next::None {
                words: crate::list::missing_folder(row),
                missing: true,
            },
            (Some(_), Some(next)) => Next::Brief {
                id: next.id.to_string(),
                size: next.size.map(|size| size.as_str().to_owned()),
                title: next.brief.title.clone(),
            },
            (Some(_), None) => Next::None {
                words: crate::list::no_next(row).to_owned(),
                missing: false,
            },
        }
    }
}

/// One badge in words (`badges::words`), its tone as the page's class.
#[derive(Debug, Clone, PartialEq)]
pub struct Badge {
    pub column: &'static str,
    pub word: String,
    pub sentence: String,
    pub tone: &'static str,
}

impl List {
    /// The list as the window's board holds it at `now`.
    pub fn of(board: &Board, now: DateTime<Utc>) -> Self {
        let portfolio = &board.portfolio;
        let filter = Filter {
            name: "",
            stacks: &NO_STACKS,
            hosting: &NO_CLASSES,
        };
        let sort: SortKey = board.view.sort;
        let groups = portfolio
            .groups(sort, filter)
            .into_iter()
            .filter_map(|group| match group.kind {
                GroupKind::State(state) => Some(Group {
                    state,
                    projects: group
                        .members
                        .iter()
                        .filter_map(|member| match member {
                            Member::Project(i) => {
                                let row = &portfolio.projects[*i];
                                let live = board.session_summary(row.name());
                                Some(Line::of(row, live.as_deref(), now))
                            }
                            Member::Folder(_) => None,
                        })
                        .collect(),
                }),
                GroupKind::Unadopted => None,
            })
            .collect();
        Self {
            reading: board.loader.reading_files && portfolio.projects.is_empty(),
            problem: portfolio.problem.clone(),
            groups,
            unadopted: portfolio
                .unadopted
                .iter()
                .map(|folder| folder.name.clone())
                .collect(),
        }
    }

    /// Every project listed.
    pub fn projects(&self) -> impl Iterator<Item = &Line> {
        self.groups.iter().flat_map(|group| group.projects.iter())
    }
}

impl Line {
    /// `row` with the window's sessions' words `live`, at `now`.
    pub fn of(row: &ProjectRow, live: Option<&str>, now: DateTime<Utc>) -> Self {
        let (messages, urgent) = row.open_messages();
        Self {
            name: row.name().to_owned(),
            state: row.project.state,
            pinned: row.project.pinned,
            proposes: row.proposes(now),
            autopilot: row.project.autopilot.enabled,
            next: Next::of(row),
            counts: row.counts(),
            owner_items: row.open_items(),
            messages,
            urgent,
            sessions: crate::list::session_words(row, live),
            usage_tenths: row.usage_tenths(),
            badges: badges::words(row, now)
                .into_iter()
                .map(|word| Badge {
                    column: word.column,
                    word: word.word,
                    sentence: word.sentence,
                    tone: Tone::class(word.tone),
                })
                .collect(),
        }
    }
}

/// The window's half: the channel's sender, the pages' ask and when it last built.
pub struct Publisher {
    sender: watch::Sender<Arc<Published>>,
    asked: Arc<AtomicBool>,
    built: Option<Instant>,
}

impl Default for Publisher {
    fn default() -> Self {
        let (sender, _) = watch::channel(Arc::new(Published::empty()));
        Self {
            sender,
            asked: Arc::new(AtomicBool::new(false)),
            built: None,
        }
    }
}

impl Publisher {
    /// The server's half, for each page to read.
    pub fn reader(&self) -> Reader {
        Reader {
            receiver: self.sender.subscribe(),
            asked: Arc::clone(&self.asked),
        }
    }

    /// Asks for a build at the next pass, as a page's ask does (EP-30a): the files landed, or a
    /// form ran.
    pub fn refresh(&self) {
        self.asked.store(true, Ordering::Release);
    }

    /// Whether this pass builds the snapshot: a page asked, or a second has passed.
    pub fn due(&self) -> bool {
        self.asked.load(Ordering::Acquire) || self.built.is_none_or(|at| at.elapsed() >= EVERY)
    }

    /// Takes the parts built at `now`: sent with a new version when they differ from the ones
    /// published, sent with their time renewed when a page asked; else their time is renewed
    /// without a word to the receivers, so that a page reads them as fresh while the window passes
    /// (the review's: `at` is the last build, not the last send).
    pub fn offer(&mut self, parts: Parts, now: DateTime<Utc>) {
        self.built = Some(Instant::now());
        let asked = self.asked.swap(false, Ordering::AcqRel);
        let Parts {
            list,
            projects,
            decisions,
        } = parts;
        self.sender.send_if_modified(|current| {
            let changed = current.list != list
                || current.projects != projects
                || current.decisions != decisions;
            let version = current.version + u64::from(changed);
            *current = Arc::new(Published {
                at: now,
                version,
                list,
                projects,
                decisions,
            });
            changed || asked
        });
    }

    /// The snapshot published last.
    #[cfg(test)]
    pub fn current(&self) -> Arc<Published> {
        Arc::clone(&self.sender.borrow())
    }
}

/// The server's half: the snapshot as last published, and the ask for a fresh one.
#[derive(Clone)]
pub struct Reader {
    receiver: watch::Receiver<Arc<Published>>,
    asked: Arc<AtomicBool>,
}

impl Reader {
    /// The version of the snapshot published last.
    pub fn version(&self) -> u64 {
        self.receiver.borrow().version
    }

    /// Waits at most `wait` for a snapshot whose version is above `version` (EP-30a): after a
    /// form's write, so that the page its redirect shows holds what the write changed (the
    /// window reads the files again after it, off its thread, and builds at most once a second).
    pub async fn newer_than(&self, version: u64, wait: Duration) {
        let mut receiver = self.receiver.clone();
        let _ = tokio::time::timeout(
            wait,
            receiver.wait_for(|published| published.version > version),
        )
        .await;
    }

    /// The snapshot for a page: the one held when it is fresh, else the window's next, asked for
    /// through `wake` and waited for at most [`WAIT`], else the one held.
    pub async fn fresh(&self, wake: impl Fn()) -> Arc<Published> {
        let mut receiver = self.receiver.clone();
        let held = Arc::clone(&receiver.borrow_and_update());
        let age = Utc::now().signed_duration_since(held.at);
        if age.to_std().is_ok_and(|age| age < FRESH_FOR) {
            return held;
        }
        self.asked.store(true, Ordering::Release);
        wake();
        match tokio::time::timeout(WAIT, receiver.changed()).await {
            Ok(Ok(())) => Arc::clone(&receiver.borrow_and_update()),
            _ => held,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::board::{Places, View};
    use crate::portfolio::tests::{Scratch, fake_store};
    use eframe::egui;

    /// A board over the fake portfolio, its files read.
    fn board(scratch: &Scratch) -> Board {
        let places = Places {
            store: fake_store(scratch),
            ledger: None,
            online: false,
            crash_log: None,
        };
        let ctx = egui::Context::default();
        let mut board = Board::new(places, &ctx, View::default(), egui::ThemePreference::Light);
        let deadline = Instant::now() + Duration::from_secs(10);
        while board.loader.reading_files && Instant::now() < deadline {
            board.receive(&ctx);
            std::thread::sleep(Duration::from_millis(20));
        }
        assert!(!board.loader.reading_files, "the files were read");
        board
    }

    #[test]
    fn the_list_says_what_the_rows_say() {
        let scratch = Scratch::new("published-list");
        let board = board(&scratch);
        let now = Utc::now();
        let list = List::of(&board, now);
        assert!(!list.reading);
        assert_eq!(list.problem, None);
        // The groups in the window's order, each project once.
        let states: Vec<State> = list.groups.iter().map(|group| group.state).collect();
        let order: Vec<State> = GroupKind::ORDER
            .iter()
            .filter_map(|kind| match kind {
                GroupKind::State(state) if states.contains(state) => Some(*state),
                _ => None,
            })
            .collect();
        assert_eq!(states, order);
        assert_eq!(list.projects().count(), board.portfolio.projects.len());
        assert_eq!(list.groups[0].heading(), "Active");
        // Each line as its row reads.
        for row in &board.portfolio.projects {
            let line = list
                .projects()
                .find(|line| line.name == row.name())
                .unwrap();
            assert_eq!(line.counts, row.counts());
            assert_eq!(line.owner_items, row.open_items());
            assert_eq!((line.messages, line.urgent), row.open_messages());
            match (row.next(), &line.next) {
                (Some(next), Next::Brief { id, title, .. }) => {
                    assert_eq!(id, &next.id.to_string());
                    assert_eq!(title, &next.brief.title);
                }
                (None, Next::None { words, .. }) => assert!(!words.is_empty()),
                (next, line) => panic!("{}: {next:?} against {line:?}", row.name()),
            }
            assert_eq!(
                line.badges.iter().any(|badge| badge.column == "Hosting"),
                row.folder.is_some(),
                "{}: a folder has its hosting",
                row.name()
            );
        }
        assert_eq!(
            list.unadopted,
            board
                .portfolio
                .unadopted
                .iter()
                .map(|folder| folder.name.clone())
                .collect::<Vec<_>>()
        );
    }

    /// Each project's page says what its Detail pane says, the decisions what the tile holds, a
    /// chat's question left to the Chat pane; a second build with nothing new is the same
    /// content, whatever the clock says (EP-30a).
    #[test]
    fn the_pages_parts_say_what_the_detail_pane_and_the_tile_say() {
        use crw_core::session::{Ended, Event, Launch, Outcome, Question};

        use crate::sessions::Live;

        let scratch = Scratch::new("published-parts");
        let mut board = board(&scratch);
        let now = Utc::now();
        let parts = Parts::of(&board, now);
        assert_eq!(parts.projects.len(), board.portfolio.projects.len());
        for row in &board.portfolio.projects {
            let page = parts
                .projects
                .iter()
                .find(|page| page.name == row.name())
                .unwrap();
            assert_eq!(
                page.waiting.len(),
                row.snapshot.open_waiting(),
                "{}",
                row.name()
            );
            assert_eq!(
                page.waiting.len() + page.resolved,
                row.snapshot.waiting().len()
            );
            assert_eq!(
                page.inbox.as_ref().map(|inbox| inbox.total),
                row.snapshot
                    .inbox
                    .as_ref()
                    .map(|inbox| inbox.messages.len())
            );
            if let (Some(next), Some(note)) = (row.next(), &page.note) {
                assert!(
                    !note.starts_with(&format!("**{}.**", next.id)),
                    "the bullet's own label is left out: {note}"
                );
            }
            assert_eq!(page.folder, row.folder.is_some());
        }
        assert!(parts.projects.iter().any(|page| page.note.is_some()));
        assert!(
            parts
                .projects
                .iter()
                .any(|page| page.waiting.iter().any(|item| item.tickable))
        );
        assert!(parts.projects.iter().any(|page| {
            page.inbox
                .as_ref()
                .is_some_and(|inbox| !inbox.open.is_empty())
        }));
        // The projects' pages and the decisions hold no age: later, they are the same.
        let later = Parts::of(&board, now + chrono::TimeDelta::minutes(3));
        assert_eq!(later.projects, parts.projects);
        assert_eq!(later.decisions, parts.decisions);
        // The window's sessions: a request waiting, a deferred run's, a chat's left out.
        let question = Question {
            text: "Which shelf?".to_owned(),
            header: "Shelf".to_owned(),
            options: Vec::new(),
            multi_select: false,
        };
        let request = |id: &str| crw_core::session::Request {
            id: id.to_owned(),
            tool: "AskUserQuestion".to_owned(),
            summary: "Which shelf?".to_owned(),
            reason: None,
            questions: vec![question.clone()],
            input: String::new(),
        };
        let launch = |project: &str| {
            let mut launch = Launch::new(project, std::path::Path::new(project), "fable", "go");
            launch.ep = Some("EP-4".to_owned());
            launch
        };
        let mut chat = launch("chat");
        chat.conversation = Some("What is next?".to_owned());
        board.sessions.live.extend([
            Live::canned(
                1,
                launch("harbor"),
                "high",
                vec![Event::Request(request("a"))],
            ),
            Live::canned(
                2,
                launch("meadow"),
                "high",
                vec![
                    Event::Request(request("b")),
                    Event::TimedOut("b".to_owned()),
                    Event::Exited(Ended {
                        outcome: Outcome::Deferred,
                        exit_code: Some(0),
                        result: None,
                        stderr: Vec::new(),
                    }),
                ],
            ),
            Live::canned(3, chat, "high", vec![Event::Request(request("c"))]),
        ]);
        let decisions = Decisions::of(&board);
        assert_eq!(decisions.waiting.len(), 1, "{decisions:?}");
        assert_eq!(
            (
                decisions.waiting[0].key,
                decisions.waiting[0].request.id.as_str()
            ),
            (1, "a")
        );
        assert_eq!(
            decisions.waiting[0].at,
            board.sessions.live[0].waiting[0].at
        );
        assert_eq!(decisions.deferred.len(), 1);
        assert_eq!(decisions.deferred[0].request.id, "b");
        assert_eq!(decisions.deferred[0].how, How::Deferred { hook: false });
        assert_eq!(decisions.count(), 2);
        // An age read from the window's clock would show here (the review's): the requests
        // backdated ten minutes, the decisions are the same.
        let old = Instant::now()
            .checked_sub(Duration::from_secs(600))
            .unwrap();
        board.sessions.live[0].waiting[0].since = old;
        if let Some(open) = board.sessions.live[1].open.as_mut() {
            open.since = old;
        }
        assert_eq!(Decisions::of(&board), decisions, "no age in it");
    }

    /// The window's own session joins the line's words, before the files' evidence.
    #[test]
    fn a_line_carries_the_sessions_words() {
        let scratch = Scratch::new("published-session");
        let board = board(&scratch);
        let row = &board.portfolio.projects[0];
        let line = Line::of(
            row,
            Some("a headless EP-4 started here at 10:00 is working"),
            Utc::now(),
        );
        assert_eq!(
            line.sessions.first().map(String::as_str),
            Some("a headless EP-4 started here at 10:00 is working")
        );
    }

    #[test]
    fn a_change_raises_the_version_and_an_ask_renews_the_time_alone() {
        let mut publisher = Publisher::default();
        let reader = publisher.reader();
        let mut listener = reader.receiver.clone();
        assert!(publisher.due(), "never built");
        assert_eq!(publisher.current().version, 0);
        let first = Utc::now();
        let list = parts(List {
            problem: Some("a problem".to_owned()),
            ..List::default()
        });
        publisher.offer(list.clone(), first);
        assert_eq!(publisher.current().version, 1);
        assert!(listener.has_changed().unwrap(), "a change is heard");
        listener.borrow_and_update();
        assert!(!publisher.due(), "built a moment ago");
        // The same content again: the time renewed, the version kept, nothing heard.
        let again = first + chrono::TimeDelta::seconds(5);
        publisher.offer(list.clone(), again);
        assert_eq!(
            (publisher.current().version, publisher.current().at),
            (1, again)
        );
        assert!(!listener.has_changed().unwrap(), "no word of a time alone");
        // A page's ask: the same version, the time renewed, heard, and due at once.
        reader.asked.store(true, Ordering::Release);
        assert!(publisher.due());
        let later = first + chrono::TimeDelta::seconds(9);
        publisher.offer(list, later);
        assert_eq!(
            (publisher.current().version, publisher.current().at),
            (1, later)
        );
        assert!(
            listener.has_changed().unwrap(),
            "the answer to an ask is heard"
        );
        listener.borrow_and_update();
        assert!(!reader.asked.load(Ordering::Acquire), "the ask was taken");
        // A change: a new version.
        publisher.offer(Parts::default(), later);
        assert_eq!(publisher.current().version, 2);
        assert!(listener.has_changed().unwrap());
        listener.borrow_and_update();
        // A change to a project's page or to the decisions alone is a change too (EP-30a).
        let mut with_decision = Parts::default();
        with_decision.decisions.elsewhere.push(Elsewhere {
            project: "harbor".to_owned(),
            mode: "visual",
            words: "waits for you in VS Code".to_owned(),
            name: None,
        });
        publisher.offer(with_decision.clone(), later);
        assert_eq!(publisher.current().version, 3);
        assert!(listener.has_changed().unwrap());
        listener.borrow_and_update();
        publisher.offer(with_decision, later);
        assert_eq!(publisher.current().version, 3, "the same parts again");
        assert!(!listener.has_changed().unwrap());
    }

    fn parts(list: List) -> Parts {
        Parts {
            list,
            ..Parts::default()
        }
    }

    /// A page with an old snapshot asks the window and takes its answer; a fresh one is taken
    /// as it is; a window that never answers leaves the page the one held.
    #[test]
    fn a_page_asks_for_a_fresh_snapshot_only_when_its_own_is_old() {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        let mut publisher = Publisher::default();
        let reader = publisher.reader();
        // Nothing published: the window is asked, and here never answers.
        let woken = std::sync::atomic::AtomicUsize::new(0);
        let started = Instant::now();
        let held = runtime.block_on(reader.fresh(|| {
            woken.fetch_add(1, Ordering::SeqCst);
        }));
        assert_eq!(woken.load(Ordering::SeqCst), 1);
        assert_eq!(held.version, 0);
        assert!(started.elapsed() >= WAIT, "it waited for the answer");
        // Published just now: taken as it is, no ask.
        publisher.offer(Parts::default(), Utc::now());
        let fresh = runtime.block_on(reader.fresh(|| panic!("a fresh snapshot needs no ask")));
        assert_eq!(fresh.version, publisher.current().version);
        // An old snapshot: the ask is answered by the next offer.
        publisher.offer(
            parts(List {
                reading: true,
                ..List::default()
            }),
            Utc::now() - chrono::TimeDelta::seconds(30),
        );
        let reader_for_page = reader.clone();
        let page = std::thread::spawn(move || {
            let runtime = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .unwrap();
            runtime.block_on(reader_for_page.fresh(|| {}))
        });
        let deadline = Instant::now() + Duration::from_secs(5);
        while !reader.asked.load(Ordering::Acquire) {
            assert!(Instant::now() < deadline, "the page never asked");
            std::thread::sleep(Duration::from_millis(5));
        }
        let answered_at = Utc::now();
        publisher.offer(
            parts(List {
                reading: true,
                ..List::default()
            }),
            answered_at,
        );
        let answer = page.join().unwrap();
        assert_eq!(
            answer.at, answered_at,
            "the window's answer, not the old one"
        );
    }
}
