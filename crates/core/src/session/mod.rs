//! The session runner (D-14 … D-16, D-18, D-29): one headless Claude Code session per [`Session`],
//! started in its project's folder with the chosen model, effort and permission mode, the
//! project's neighbour folders read-only, a pre-assigned session id and the prompt on stdin; its
//! stream-json read into [`Event`]s through the adapter ([`protocol`]); its requests answered by
//! the host; stopped gracefully or killed with its whole process tree; recorded in the ledger. The
//! child runs in a job object the system kills when the tool lets go of it, so no session outlives
//! the tool or leaves a process behind. Each session has a thread with a small tokio runtime of its
//! own, and its caller sees a channel of events and a handle, which suits the window's frame loop
//! and `crw` alike. Tests drive the stub `stub-claude`, never the real CLI (D-29).

pub mod cli;
mod launch;
pub mod protocol;

use std::collections::VecDeque;
use std::io;
use std::path::{Path, PathBuf};
use std::process::{ExitStatus, Stdio};
use std::sync::mpsc;
use std::sync::{Mutex, PoisonError};
use std::thread;
use std::time::{Duration, Instant};

use chrono::{DateTime, Utc};
use process_wrap::tokio::{ChildWrapper, CommandWrap, KillOnDrop};
#[cfg(windows)]
use process_wrap::tokio::{CreationFlags, JobObject};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{ChildStdin, Command};
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender, unbounded_channel};

pub use cli::Preflight;
pub(crate) use launch::markers;
pub use launch::{Launch, NEIGHBOUR_CLAUDE_MD, STOP_TIMEOUT, edit_deny_rule};
pub use protocol::{
    Choice, Denied, Finish, Init, Question, Request, Response, ToolResult, ToolUse, Unreadable,
};

use crate::ledger::{Finished, Ledger, NewSession};
use crate::process;
use launch::CommandLine;
use protocol::Protocol;

/// The least time between two session starts (D-29).
pub const SPAWN_SPACING: Duration = Duration::from_secs(2);

/// How many of stderr's last lines an ending keeps.
const STDERR_LINES: usize = 20;

/// How long the runner reads what is left of a session's output once it has exited.
const DRAIN: Duration = Duration::from_secs(5);

/// When the last session started, for the spacing.
static LAST_START: Mutex<Option<Instant>> = Mutex::new(None);

/// What a session said, or what became of it, in the order it happened; [`Event::Exited`] comes
/// last.
#[derive(Debug, Clone, PartialEq)]
pub enum Event {
    /// The CLI's own account of the run: version, model, effort, mode, tools, capabilities.
    Init(Init),
    /// The control protocol's handshake answered (D-19): EP-14 chooses its fallback from it.
    Handshake(Result<(), String>),
    /// A piece of the assistant's text as it streams.
    TextDelta(String),
    /// A whole text block of an assistant message.
    Text(String),
    ToolUse(ToolUse),
    /// What a tool call came back with, its first lines (EP-13).
    ToolResult(ToolResult),
    /// The session waits for the host's [`Response`] (D-19).
    Request(Request),
    /// The session no longer waits for the answer to this request.
    Withdrawn(String),
    /// A tool use refused without asking (D-18 addendum).
    Denied(Denied),
    Result(Finish),
    /// Something the runner did or saw on the session's behalf, in words.
    Notice(String),
    Unreadable(Unreadable),
    Exited(Ended),
}

/// How a run ended, in the ledger's words (D-26).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Outcome {
    Completed,
    /// The host stopped or killed it before its result.
    Stopped,
    Failed,
    /// It waits on a deferred tool use, to be resumed (D-19).
    Deferred,
}

impl Outcome {
    pub fn as_str(self) -> &'static str {
        match self {
            Outcome::Completed => "completed",
            Outcome::Stopped => "stopped",
            Outcome::Failed => "failed",
            Outcome::Deferred => "deferred",
        }
    }
}

/// What a run came to.
#[derive(Debug, Clone, PartialEq)]
pub struct Ended {
    pub outcome: Outcome,
    /// The process's exit code; none when it could not be read.
    pub exit_code: Option<i32>,
    /// The result, read even when the process exited with an error.
    pub result: Option<Finish>,
    /// The last lines the session wrote to stderr.
    pub stderr: Vec<String>,
}

/// A running headless session. Dropping the handle kills the session with its process tree.
pub struct Session {
    /// The id the session was started with (`--session-id`), by which it resumes.
    pub session_id: String,
    /// The ledger's row for the run, when it is recorded.
    pub ledger_row: Option<i64>,
    /// What the launch asked for and the CLI cannot take, in words.
    pub dropped: Vec<String>,
    events: mpsc::Receiver<Event>,
    orders: UnboundedSender<Order>,
}

/// What the host tells a session's thread.
enum Order {
    Respond(String, Response),
    Stop,
    Kill,
}

impl Session {
    /// Starts a headless run (D-16) after the spacing since the last start (D-29), or says why
    /// not: the preflight's refusals, or a launch the CLI cannot run. The child is running when
    /// this returns.
    pub fn start(preflight: &Preflight, launch: Launch) -> Result<Self, String> {
        let refusals = preflight.refusals();
        if !refusals.is_empty() {
            return Err(refusals.join("; "));
        }
        let flags = preflight.flags.as_ref().map_err(Clone::clone)?;
        // A resumed run goes on under the session's own id (EP-13).
        let session_id = launch
            .resume
            .clone()
            .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
        let line = launch::command_line(
            &launch,
            flags,
            &session_id,
            std::env::vars_os().map(|(name, _)| name),
        )?;
        let dropped = line.dropped.clone();
        let (events_tx, events) = mpsc::channel();
        let (orders, orders_rx) = unbounded_channel();
        let (started_tx, started) = mpsc::sync_channel(1);
        let job = Job {
            program: preflight.program.clone(),
            line,
            launch,
            session_id: session_id.clone(),
        };

        let _turn = process::turn();
        let last = *LAST_START.lock().unwrap_or_else(PoisonError::into_inner);
        if let Some(since) = last.map(|at| at.elapsed())
            && since < SPAWN_SPACING
        {
            thread::sleep(SPAWN_SPACING - since);
        }
        thread::Builder::new()
            .name(format!("session {}", job.launch.name()))
            .spawn(move || job.run(events_tx, orders_rx, started_tx))
            .map_err(|e| format!("could not start the session's thread: {e}"))?;
        let started = started
            .recv()
            .map_err(|_| "the session's thread ended before its child started".to_owned());
        *LAST_START.lock().unwrap_or_else(PoisonError::into_inner) = Some(Instant::now());
        Ok(Self {
            session_id,
            ledger_row: started??,
            dropped,
            events,
            orders,
        })
    }

    /// The session's events; the last is [`Event::Exited`].
    pub fn events(&self) -> &mpsc::Receiver<Event> {
        &self.events
    }

    /// Answers a [`Request`]; `false` when the session has ended.
    pub fn respond(&self, request_id: &str, response: Response) -> bool {
        self.orders
            .send(Order::Respond(request_id.to_owned(), response))
            .is_ok()
    }

    /// Stops gracefully: the interrupt ends the turn, stdin closes, and the job is killed if the
    /// session has not exited within the launch's stop timeout. `false` when it has ended.
    pub fn stop(&self) -> bool {
        self.orders.send(Order::Stop).is_ok()
    }

    /// Kills the session's whole process tree at once; `false` when it has ended.
    pub fn kill(&self) -> bool {
        self.orders.send(Order::Kill).is_ok()
    }
}

/// What a session's thread needs.
struct Job {
    program: PathBuf,
    line: CommandLine,
    launch: Launch,
    session_id: String,
}

impl Job {
    /// The session's thread: starts the child, reports it to [`Session::start`], records it,
    /// watches it to its end.
    fn run(
        self,
        events: mpsc::Sender<Event>,
        orders: UnboundedReceiver<Order>,
        started: mpsc::SyncSender<Result<Option<i64>, String>>,
    ) {
        let runtime = match tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
        {
            Ok(runtime) => runtime,
            Err(e) => {
                let _ = started.send(Err(format!("could not make the session's runtime: {e}")));
                return;
            }
        };
        runtime.block_on(async {
            let child = match spawn(&self.program, &self.line, &self.launch.folder) {
                Ok(child) => child,
                Err(e) => {
                    let _ = started.send(Err(format!(
                        "could not start {} in {}: {e}",
                        self.program.display(),
                        self.launch.folder.display()
                    )));
                    return;
                }
            };
            let row = self.record_start(Utc::now());
            let _ = started.send(Ok(row.as_ref().ok().copied().flatten()));
            if let Err(e) = &row {
                let _ = events.send(Event::Notice(e.clone()));
            }
            let (ended, questions) = watch(child, &self.launch, &events, orders).await;
            if let Ok(Some(row)) = row
                && let Err(e) = self.record_end(row, &ended, questions)
            {
                let _ = events.send(Event::Notice(e));
            }
            let _ = events.send(Event::Exited(ended));
        });
        runtime.shutdown_timeout(Duration::from_secs(1));
    }

    /// The run's ledger row, from its start: none when no ledger was asked for, `Err` saying why
    /// the ledger did not take it.
    fn record_start(&self, started_at: DateTime<Utc>) -> Result<Option<i64>, String> {
        let Some(path) = &self.launch.ledger else {
            return Ok(None);
        };
        let launch = &self.launch;
        Ledger::open(path)
            .and_then(|ledger| {
                ledger.start_session(&NewSession {
                    session_id: Some(&self.session_id),
                    project: &launch.project,
                    ep: launch.ep.as_deref(),
                    mode: "headless",
                    model: Some(&launch.model),
                    effort: self.line.effort.as_deref(),
                    ultracode: self.line.effort.as_deref() == Some("ultracode"),
                    started_at,
                })
            })
            .map(Some)
            .map_err(|e| format!("the ledger did not record the session: {e}"))
    }

    fn record_end(&self, row: i64, ended: &Ended, questions: u32) -> Result<(), String> {
        let Some(path) = &self.launch.ledger else {
            return Ok(());
        };
        let result = ended.result.as_ref();
        Ledger::open(path)
            .and_then(|ledger| {
                ledger.finish_session(
                    row,
                    &Finished {
                        at: Utc::now(),
                        outcome: ended.outcome.as_str(),
                        cost_usd: result.map(|result| result.cost_usd),
                        questions,
                        result: result.and_then(|result| result.text.as_deref()),
                    },
                )
            })
            .map_err(|e| format!("the ledger did not record the session's end: {e}"))
    }
}

/// The child, in the project's folder, without a console window, in a job object that is killed
/// when its handle closes (D-14, D-29).
fn spawn(program: &Path, line: &CommandLine, folder: &Path) -> io::Result<Box<dyn ChildWrapper>> {
    let mut command = Command::new(program);
    command
        .args(&line.args)
        .current_dir(folder)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    for (name, value) in &line.env {
        match value {
            Some(value) => command.env(name, value),
            None => command.env_remove(name),
        };
    }
    let mut wrap = CommandWrap::from(command);
    wrap.wrap(KillOnDrop);
    #[cfg(windows)]
    {
        use windows::Win32::System::Threading::CREATE_NO_WINDOW;
        wrap.wrap(CreationFlags(CREATE_NO_WINDOW));
        wrap.wrap(JobObject);
    }
    wrap.spawn()
}

/// What the watch keeps as the lines go by.
#[derive(Default)]
struct Watch {
    result: Option<Finish>,
    stderr: VecDeque<String>,
    questions: u32,
    stopping: bool,
    /// The host stopped the run before its result.
    stopped_early: bool,
}

impl Watch {
    fn stop(&mut self) {
        if !self.stopping {
            self.stopping = true;
            self.stopped_early = self.result.is_none();
        }
    }

    fn stderr(&mut self, line: String) {
        if self.stderr.len() == STDERR_LINES {
            self.stderr.pop_front();
        }
        self.stderr.push_back(line);
    }

    /// Reads one stdout line: answers what must be answered at once, keeps the result and counts
    /// the questions, passes the events on. After the result stdin closes, so the session exits:
    /// a run is one prompt, and a follow-up is a run of its own that resumes it by id (EP-13).
    async fn read(
        &mut self,
        protocol: &mut Protocol,
        line: &str,
        stdin: &mut Option<ChildStdin>,
        events: &mpsc::Sender<Event>,
    ) {
        let reading = protocol.read(line);
        if let Some(reply) = &reading.reply {
            send(stdin, reply).await;
        }
        for event in reading.events {
            match &event {
                Event::Request(request) if !request.questions.is_empty() => self.questions += 1,
                Event::Result(finish) => {
                    self.result = Some(finish.clone());
                    *stdin = None;
                }
                _ => {}
            }
            let _ = events.send(event);
        }
    }

    fn ended(self, status: Option<ExitStatus>) -> Ended {
        let outcome = match &self.result {
            _ if self.stopped_early => Outcome::Stopped,
            Some(result) if result.is_deferred() => Outcome::Deferred,
            Some(result) if result.succeeded() => Outcome::Completed,
            _ => Outcome::Failed,
        };
        Ended {
            outcome,
            exit_code: status.and_then(|status| status.code()),
            result: self.result,
            stderr: self.stderr.into(),
        }
    }
}

/// Watches the child to its end: the handshake and the prompt first, then its output, the host's
/// orders, the stop's deadline and its exit, whichever comes; then what is left of its output.
/// Returns how it ended and how many questions it asked.
async fn watch(
    mut child: Box<dyn ChildWrapper>,
    launch: &Launch,
    events: &mpsc::Sender<Event>,
    mut orders: UnboundedReceiver<Order>,
) -> (Ended, u32) {
    let mut stdin = child.stdin().take();
    let mut lines = BufReader::new(child.stdout().take().expect("stdout is piped")).lines();
    let mut errors = BufReader::new(child.stderr().take().expect("stderr is piped")).lines();
    let mut protocol = Protocol::default();
    let mut watch = Watch::default();
    let opening = protocol.initialize() + &Protocol::prompt(&launch.prompt);
    send(&mut stdin, &opening).await;

    let (mut reading, mut erroring, mut listening) = (true, true, true);
    let mut deadline: Option<tokio::time::Instant> = None;
    let status = loop {
        tokio::select! {
            line = lines.next_line(), if reading => match line {
                Ok(Some(line)) => watch.read(&mut protocol, &line, &mut stdin, events).await,
                _ => reading = false,
            },
            line = errors.next_line(), if erroring => match line {
                Ok(Some(line)) => watch.stderr(line),
                _ => erroring = false,
            },
            order = orders.recv(), if listening => match order {
                Some(Order::Respond(id, response)) => match protocol.answer(&id, &response) {
                    Ok(answer) => send(&mut stdin, &answer).await,
                    Err(e) => notice(events, e),
                },
                Some(Order::Stop) => {
                    if !watch.stopping {
                        watch.stop();
                        let interrupt = protocol.interrupt();
                        send(&mut stdin, &interrupt).await;
                        stdin = None;
                        deadline = Some(tokio::time::Instant::now() + launch.stop_timeout);
                    }
                }
                Some(Order::Kill) => {
                    watch.stop();
                    kill(&mut child, events);
                }
                // The handle was dropped: nothing waits for the session any more (D-29).
                None => {
                    listening = false;
                    watch.stop();
                    kill(&mut child, events);
                }
            },
            () = tokio::time::sleep_until(deadline.unwrap_or_else(tokio::time::Instant::now)),
                if deadline.is_some() =>
            {
                deadline = None;
                notice(
                    events,
                    format!("no exit within {:?} of the stop: the job was killed", launch.stop_timeout),
                );
                kill(&mut child, events);
            },
            status = child.wait() => break status.ok(),
        }
    };
    // The main process is gone: what it left in its job goes too, so that nothing holds the pipes
    // open, then what remains of the output is read.
    let _ = child.start_kill();
    let drain = async {
        while reading {
            match lines.next_line().await {
                Ok(Some(line)) => watch.read(&mut protocol, &line, &mut stdin, events).await,
                _ => reading = false,
            }
        }
        while let Ok(Some(line)) = errors.next_line().await {
            watch.stderr(line);
        }
    };
    if tokio::time::timeout(DRAIN, drain).await.is_err() {
        notice(
            events,
            "the session's output did not end after its exit".to_owned(),
        );
    }
    let questions = watch.questions;
    (watch.ended(status), questions)
}

/// Writes a line to the session's stdin; a pipe that fails is dropped, since the session is gone
/// or going.
async fn send(stdin: &mut Option<ChildStdin>, text: &str) {
    let failed = match stdin {
        Some(pipe) => pipe.write_all(text.as_bytes()).await.is_err() || pipe.flush().await.is_err(),
        None => false,
    };
    if failed {
        *stdin = None;
    }
}

fn kill(child: &mut Box<dyn ChildWrapper>, events: &mpsc::Sender<Event>) {
    if let Err(e) = child.start_kill() {
        notice(events, format!("could not kill the session's job: {e}"));
    }
}

fn notice(events: &mpsc::Sender<Event>, text: String) {
    let _ = events.send(Event::Notice(text));
}
