//! The loopback server (D-14, D-20, D-28; EP-12): axum on 127.0.0.1 at the registry's port, on a
//! thread with a small runtime of its own, so the frame loop sees a channel. `POST /hook` and
//! `POST /statusline` take what `crw hook` and `crw statusline` relay and hand it to the window
//! with a repaint; `GET /health` names the app (single instance probes it at EP-28). Loopback
//! callers are trusted (D-28), and only they get through: a request that carries an `Origin` (a web
//! page's) or names another host than the loopback (a page whose name was pointed at 127.0.0.1) is
//! refused, and a body is JSON, which a page cannot send across origins without asking first.
//! `POST /restart` (EP-37) is `crw install`'s request that the window restart into the installed
//! build: the window answers it at its next pass, and the server waits for that answer a few
//! seconds.
//!
//! The phone's pages (D-28; EP-30) are under `/m/`: `GET /m/` the project list, `GET /m/app.css`
//! their stylesheet, `GET /m` a redirect to the list, any other `/m/…` path a page not found. They
//! read what the window publishes (`published`), never the window itself. In the loopback guard's
//! place, a middleware applies `crw_core::remote`'s identity check to every one of them: local by
//! the loopback `Host` with no login and no proxy's header, through Serve only with the owner's
//! login, read from the registry at that request; a foreign `Origin` refused, and a form naming its
//! own. An action through Serve is written to the ledger before it runs, a refusal at most once a
//! minute for each identity, and every page answers with headers that keep it out of caches and
//! frames and let it load only its own stylesheet.
//!
//! EP-30a adds a project's page (`GET /m/p/<project>`) with its three forms (an owner task ticked,
//! an inbox message ticked with its disposition, a message sent), and the decisions page (`GET
//! /m/decisions`) with its answer form. A form's handler hands the window a [`PageRequest`]
//! through a channel drained in the window's pass, shown or hidden, waits at most [`FORM_WAIT`]
//! for what came of it, keeps that line under a number and answers a 303 to its page with
//! `?done=<n>`, so a reload never posts again and a page never draws words a link carries.
//!
//! EP-30b adds the actions that run things and the live updates: a project's start page (`GET
//! /m/p/<project>/start`, the window's preview asked through a [`StartAsk`]) whose form starts
//! the next brief headless, a run's Stop on the project's page, the autopilot's Pause and Resume
//! on the list's; the event stream (`GET /m/events`), whose `change` events name the path of each
//! page whose part changed, and the tool's own script that listens to it (`GET /m/app.js`).

use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::convert::Infallible;
use std::net::Ipv4Addr;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use axum::body::Bytes;
use axum::extract::{DefaultBodyLimit, Path, Request, State};
use axum::http::{HeaderMap, HeaderValue, StatusCode, Uri, header};
use axum::middleware::{self, Next};
use axum::response::sse::{Event, KeepAlive, Sse};
use axum::response::{Html, IntoResponse, Redirect, Response};
use axum::routing::{any, get, post};
use axum::{Extension, Json, Router};
use crw_core::ledger::Ledger;
use crw_core::registry::Store;
use crw_core::remote::{self, Asked, Caller, Throttle};
use crw_core::signals::{self, Hook, RestartAnswer, RestartAsk, Signal, StatusLine};
use futures_util::StreamExt;
use serde::Serialize;
use tokio::sync::oneshot;

use crate::decisions::Verb;
use crate::form;
use crate::published;

/// The largest body taken: the relays' forms are a few hundred bytes.
const BODY_LIMIT: usize = 256 << 10;

/// How long a restart request waits for the window's answer, within `crw install`'s budget.
const RESTART_WAIT: Duration = Duration::from_secs(6);

/// The phone's pages' paths (EP-30): the list, its bare form, the stylesheet, and the rest.
pub const PAGES_ROOT: &str = "/m/";
const PAGES_BARE: &str = "/m";
const STYLESHEET_ROUTE: &str = "/m/app.css";
const PAGES_OTHER: &str = "/m/{*rest}";

/// A project's page and its forms (EP-30a), and the decisions page with its form.
pub const PROJECT_PREFIX: &str = "/m/p/";
const PROJECT_ROUTE: &str = "/m/p/{project}";
const TICK_ROUTE: &str = "/m/p/{project}/tick";
const MESSAGE_ROUTE: &str = "/m/p/{project}/message";
const SEND_ROUTE: &str = "/m/p/{project}/send";
pub const DECISIONS_ROUTE: &str = "/m/decisions";
const ANSWER_ROUTE: &str = "/m/decisions/answer";

/// The actions that run things (EP-30b): a project's start page and its form, a run's Stop, the
/// autopilot's Pause and Resume; the event stream and the script that listens to it.
const START_ROUTE: &str = "/m/p/{project}/start";
const STOP_ROUTE: &str = "/m/p/{project}/stop";
pub const AUTOPILOT_ROUTE: &str = "/m/autopilot";
pub const EVENTS_ROUTE: &str = "/m/events";
pub const SCRIPT_ROUTE: &str = "/m/app.js";

/// How long the start page waits for the window's preview (EP-30b).
const PREVIEW_WAIT: Duration = Duration::from_secs(5);

/// How long the browser waits before it opens the event stream again (EP-30b): the window's
/// restart takes a few seconds.
const RETRY: Duration = Duration::from_secs(5);

/// How long a form waits for the window's answer (EP-30a): a message's write waits for the
/// process's turn, which another pass's git child may hold a moment.
const FORM_WAIT: Duration = Duration::from_secs(10);

/// How long a form that changed something waits for the window's snapshot to show it (EP-30a).
const SHOWN_WAIT: Duration = Duration::from_secs(3);

/// How long a form's outcome is kept for its page, and how many are kept (EP-30a).
const SAID_FOR: Duration = Duration::from_secs(300);
const SAID_KEPT: usize = 32;

/// What a page may load: its own stylesheet, its own forms, and from EP-30b its own script and
/// the window's own event stream and pages for that script to fetch; nothing else.
const CONTENT_POLICY: &str = "default-src 'none'; style-src 'self'; script-src 'self'; \
                              connect-src 'self'; form-action 'self'; frame-ancestors 'none'; \
                              base-uri 'none'";

/// What the server needs for the phone's pages (EP-30): the window's published snapshot, the
/// registry the owner's login is read from, and the ledger the remote-action log is written to.
#[derive(Clone)]
pub struct Pages {
    pub published: published::Reader,
    pub store: Arc<Store>,
    pub ledger: Option<PathBuf>,
}

/// A restart request from the loopback (EP-37), waiting for the window's answer: who sends it,
/// the install folder it wrote, and where the answer goes, closed once the caller stopped
/// waiting.
pub struct RestartRequest {
    pub from: String,
    pub bin: Option<std::path::PathBuf>,
    pub reply: oneshot::Sender<RestartAnswer>,
}

/// What a page's form asks the window to do (D-28; EP-30a): the phone's ticks, its message and
/// its answers, each run on the window's thread.
#[derive(Debug, Clone, PartialEq)]
pub enum Order {
    /// An owner task ticked (D-10).
    TickTask { project: String, id: String },
    /// An inbox message ticked with the disposition the form carries (D-13).
    TickMessage {
        project: String,
        id: String,
        disposition: String,
    },
    /// The owner's message written into the project's inbox (D-13; EP-20c).
    Send {
        project: String,
        text: String,
        urgent: bool,
    },
    /// A request the window's session `key` waits on, or its deferred run's, answered (D-19).
    Answer {
        key: u64,
        request: String,
        form: AnswerForm,
    },
    /// The project's next brief started headless with the rules' choice (D-28, D-22; EP-30b):
    /// `ep` is the brief the start page showed, refused when it is no longer the next.
    Start { project: String, ep: String },
    /// The window's run `key` of the project's `ep` stopped gracefully, never killed (EP-30b).
    Stop {
        project: String,
        key: u64,
        ep: String,
    },
    /// Every chain paused by the owner, or resumed (D-22; EP-30b): said, never a toggle.
    Pause { paused: bool },
}

/// The start page's ask for the window's preview of a project's start (EP-30b): a draft reads
/// the project's files and asks the CLI, which the snapshot does not hold. None comes back for a
/// project the window does not list.
pub struct StartAsk {
    pub project: String,
    pub reply: oneshot::Sender<Option<published::Start>>,
}

/// An answer form as the page sent it: its button, the options picked and the owner's own words
/// by question, the reason. The window builds the response from it by the tile's rules, against
/// the request as it holds it.
#[derive(Debug, Clone, PartialEq)]
pub struct AnswerForm {
    pub verb: Verb,
    pub picks: BTreeMap<usize, BTreeSet<usize>>,
    pub other: BTreeMap<usize, String>,
    pub reason: String,
}

/// What came of a form: done, in words, or why not.
pub type Said = Result<String, String>;

/// A page's form waiting for the window's hand (EP-30a): the order, who sent it in the window's
/// words, where the outcome goes (closed once the page stopped waiting), and where it stands
/// between the two.
pub struct PageRequest {
    pub order: Order,
    pub caller: String,
    pub reply: oneshot::Sender<Said>,
    pub hand: Hand,
}

/// Where a form stands between its page and the window (EP-30a's review): waiting, taken by the
/// window, or let go by its page at its ten seconds. Each side moves it once, so a form is never
/// both run and said not done, and a page that let go of a form the window had taken says so: a
/// message's write goes on after the page's ten seconds.
#[derive(Clone, Default)]
pub struct Hand(Arc<AtomicU8>);

impl Hand {
    const WAITING: u8 = 0;
    const TAKEN: u8 = 1;
    const LET_GO: u8 = 2;

    /// The window takes the form: false when its page let it go first.
    pub fn take(&self) -> bool {
        self.0
            .compare_exchange(
                Self::WAITING,
                Self::TAKEN,
                Ordering::AcqRel,
                Ordering::Acquire,
            )
            .is_ok()
    }

    /// The page lets the form go: false when the window took it first.
    pub(crate) fn let_go(&self) -> bool {
        self.0
            .compare_exchange(
                Self::WAITING,
                Self::LET_GO,
                Ordering::AcqRel,
                Ordering::Acquire,
            )
            .is_ok()
    }
}

/// The forms' outcomes kept for their pages (EP-30a), by number and page: a redirect names the
/// number, so a page never draws words that a link carries, and only the page the form came back
/// to draws its line. The numbers start from the process's start time in milliseconds, so a page
/// reloaded after the window's restart draws no other form's line (the review's).
struct Outcomes {
    last: u64,
    kept: VecDeque<(u64, String, Said, Instant)>,
}

impl Default for Outcomes {
    fn default() -> Self {
        let started = chrono::Utc::now().timestamp_millis();
        Self {
            last: u64::try_from(started).unwrap_or_default(),
            kept: VecDeque::new(),
        }
    }
}

impl Outcomes {
    fn put(&mut self, path: &str, said: Said) -> u64 {
        self.last += 1;
        self.kept
            .push_back((self.last, path.to_owned(), said, Instant::now()));
        while self.kept.len() > SAID_KEPT {
            self.kept.pop_front();
        }
        self.last
    }

    fn get(&self, number: u64, path: &str) -> Option<Said> {
        self.kept
            .iter()
            .find(|(n, page, _, at)| *n == number && page == path && at.elapsed() < SAID_FOR)
            .map(|(_, _, said, _)| said.clone())
    }
}

/// What the window is woken with when the server has news.
type Wake = Arc<dyn Fn() + Send + Sync>;

/// Where the server stands.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Listening {
    Starting(u16),
    On(u16),
    /// The port is taken: another window, or something else (EP-40's guard asks).
    Taken(u16, String),
    /// The socket could not be made, in words.
    Failed(String),
}

/// The server's handle: what it heard, and whether it listens.
pub struct Server {
    signals: Receiver<Signal>,
    restarts: Receiver<RestartRequest>,
    orders: Receiver<PageRequest>,
    asks: Receiver<StartAsk>,
    bound: Receiver<Result<u16, Listening>>,
    pub listening: Listening,
}

struct Shared {
    port: u16,
    sender: Sender<Signal>,
    restarts: Sender<RestartRequest>,
    /// The pages' forms, for the window's hand (EP-30a).
    orders: Sender<PageRequest>,
    /// The start pages' asks for the window's preview (EP-30b).
    asks: Sender<StartAsk>,
    wake: Wake,
    /// The phone's pages' needs; none in the relays' tests, whose `/m/` pages say they are not
    /// served.
    pages: Option<Pages>,
    /// The refused identities' last lines (EP-30).
    refused: Mutex<Throttle>,
    /// The forms' outcomes for their pages (EP-30a).
    outcomes: Mutex<Outcomes>,
}

impl Server {
    /// Starts listening on `127.0.0.1:<port>` (0 for any free port, in the tests); `wake` asks the
    /// window for a frame. The phone's pages are not served: the relays' tests.
    #[cfg(test)]
    pub fn start(port: u16, wake: impl Fn() + Send + Sync + 'static) -> Server {
        Server::serve(port, None, wake)
    }

    /// Starts listening on `127.0.0.1:<port>` (0 for any free port, in the tests), the phone's
    /// pages served from `pages` (EP-30); `wake` asks the window for a frame.
    pub fn serve(
        port: u16,
        pages: Option<Pages>,
        wake: impl Fn() + Send + Sync + 'static,
    ) -> Server {
        let (sender, signals) = mpsc::channel();
        let (restart_sender, restarts) = mpsc::channel();
        let (order_sender, orders) = mpsc::channel();
        let (ask_sender, asks) = mpsc::channel();
        let (bound_sender, bound) = mpsc::channel();
        let wake: Wake = Arc::new(wake);
        let spawned = thread::Builder::new()
            .name("signals server".to_owned())
            .spawn(move || {
                let runtime = match tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                {
                    Ok(runtime) => runtime,
                    Err(e) => {
                        let _ =
                            bound_sender.send(Err(Listening::Failed(format!("no runtime: {e}"))));
                        wake();
                        return;
                    }
                };
                runtime.block_on(async move {
                    let listener =
                        match tokio::net::TcpListener::bind((Ipv4Addr::LOCALHOST, port)).await {
                            Ok(listener) => listener,
                            Err(e) => {
                                let why = format!("127.0.0.1:{port}: {e}");
                                let state = if e.kind() == std::io::ErrorKind::AddrInUse {
                                    Listening::Taken(port, why)
                                } else {
                                    Listening::Failed(why)
                                };
                                let _ = bound_sender.send(Err(state));
                                wake();
                                return;
                            }
                        };
                    let actual = listener.local_addr().map_or(port, |address| address.port());
                    let _ = bound_sender.send(Ok(actual));
                    wake();
                    let shared = Arc::new(Shared {
                        port: actual,
                        sender,
                        restarts: restart_sender,
                        orders: order_sender,
                        asks: ask_sender,
                        wake,
                        pages,
                        refused: Mutex::new(Throttle::default()),
                        outcomes: Mutex::new(Outcomes::default()),
                    });
                    // It serves until the window closes, which ends the process.
                    let _ = axum::serve(listener, router(shared)).await;
                });
            });
        let listening = match spawned {
            Ok(_) => Listening::Starting(port),
            Err(e) => Listening::Failed(format!("no thread for the server: {e}")),
        };
        Server {
            signals,
            restarts,
            orders,
            asks,
            bound,
            listening,
        }
    }

    /// Takes in where the server stands and what it heard since the last frame.
    pub fn receive(&mut self) -> Vec<Signal> {
        if let Ok(bound) = self.bound.try_recv() {
            self.listening = match bound {
                Ok(port) => Listening::On(port),
                Err(state) => state,
            };
        }
        self.signals.try_iter().collect()
    }

    /// The restart requests waiting for the window's answer (EP-37).
    pub fn take_restarts(&mut self) -> Vec<RestartRequest> {
        self.restarts.try_iter().collect()
    }

    /// The pages' forms waiting for the window's hand (EP-30a).
    pub fn take_orders(&mut self) -> Vec<PageRequest> {
        self.orders.try_iter().collect()
    }

    /// The start pages' asks for the window's preview (EP-30b).
    pub fn take_asks(&mut self) -> Vec<StartAsk> {
        self.asks.try_iter().collect()
    }
}

fn router(shared: Arc<Shared>) -> Router {
    // The phone's pages, each behind the identity check (EP-30), and their forms (EP-30a).
    let pages = Router::new()
        .route(PAGES_BARE, get(to_list))
        .route(PAGES_ROOT, get(list_page))
        .route(STYLESHEET_ROUTE, get(stylesheet))
        .route(PROJECT_ROUTE, get(project_page))
        .route(TICK_ROUTE, post(tick_task))
        .route(MESSAGE_ROUTE, post(tick_message))
        .route(SEND_ROUTE, post(send_message))
        .route(DECISIONS_ROUTE, get(decisions_page))
        .route(ANSWER_ROUTE, post(answer))
        // The actions that run things and the live updates (EP-30b).
        .route(START_ROUTE, get(start_page).post(start))
        .route(STOP_ROUTE, post(stop))
        .route(AUTOPILOT_ROUTE, post(pause))
        .route(EVENTS_ROUTE, get(events))
        .route(SCRIPT_ROUTE, get(script))
        .route(PAGES_OTHER, any(not_found))
        .layer(middleware::from_fn_with_state(Arc::clone(&shared), guard));
    Router::new()
        .route(signals::HOOK_ROUTE, post(hook))
        .route(signals::STATUS_ROUTE, post(status_line))
        .route(signals::HEALTH_ROUTE, get(health))
        .route(signals::RESTART_ROUTE, post(restart))
        .route(signals::SHOW_ROUTE, post(show))
        .merge(pages)
        .layer(DefaultBodyLimit::max(BODY_LIMIT))
        .with_state(shared)
}

/// The identity check of every page (D-28; EP-30): a request that passes goes on with its caller
/// among its extensions, an action through Serve once its line is written; a refused one gets a
/// page that names no reason, its line written at most once a minute for its identity.
async fn guard(State(shared): State<Arc<Shared>>, mut request: Request, next: Next) -> Response {
    let asked = Asked::new(
        request.method().as_str(),
        request.uri().path(),
        request
            .headers()
            .iter()
            .map(|(name, value)| (name.as_str(), value.to_str().unwrap_or_default())),
    );
    let owner = || {
        let pages = shared
            .pages
            .as_ref()
            .ok_or_else(|| "the pages are not served".to_owned())?;
        pages
            .store
            .load()
            .map(|loaded| loaded.registry.settings.owner_login)
            .map_err(|e| e.to_string())
    };
    match remote::check(&asked, shared.port, owner) {
        Ok(caller) => {
            if caller.is_remote() && !asked.is_safe() {
                let line = remote::allowed_line(&caller, &asked);
                if !shared.log(line).await {
                    return secured(page(
                        StatusCode::SERVICE_UNAVAILABLE,
                        "Not done",
                        "The action log could not be written, so nothing was done.",
                    ));
                }
            }
            request.extensions_mut().insert(caller);
            secured(next.run(request).await)
        }
        Err(refusal) => {
            let identity = remote::refused_identity(&asked);
            let due = shared
                .refused
                .lock()
                .is_ok_and(|mut throttle| throttle.due(&identity, Instant::now()));
            if due {
                shared.log_detached(remote::refused_line(&asked, &refusal));
            }
            secured(page(
                StatusCode::FORBIDDEN,
                "Not allowed",
                "This page answers its owner only.",
            ))
        }
    }
}

/// The headers every page answers with: never cached, never sniffed or framed, the referrer
/// kept to the page's own origin (so a form still names it), and nothing loaded but the page's
/// own stylesheet.
fn secured(mut response: Response) -> Response {
    let headers = response.headers_mut();
    for (name, value) in [
        (header::CACHE_CONTROL, "no-store"),
        (header::X_CONTENT_TYPE_OPTIONS, "nosniff"),
        (header::REFERRER_POLICY, "same-origin"),
        (header::X_FRAME_OPTIONS, "DENY"),
        (header::CONTENT_SECURITY_POLICY, CONTENT_POLICY),
    ] {
        headers.insert(name, HeaderValue::from_static(value));
    }
    response
}

/// A page of one heading and one line, with its status.
fn page(status: StatusCode, heading: &str, words: &str) -> Response {
    (status, Html(crate::pages::message(heading, words))).into_response()
}

/// `GET /m`: the list is at `/m/`, named by its path alone so that it holds through Serve.
async fn to_list() -> Redirect {
    Redirect::to(PAGES_ROOT)
}

/// `GET /m/`: the project list from the window's snapshot, a fresh one asked for when the one
/// held is old, with the outcome of the Pause or Resume that led here (EP-30b).
async fn list_page(State(shared): State<Arc<Shared>>, uri: Uri) -> Response {
    let Some(pages) = &shared.pages else {
        return not_served();
    };
    let published = pages.published.fresh(|| (shared.wake)()).await;
    let said = shared.said(&uri);
    match crate::pages::list(&published, said.as_ref()) {
        Ok(html) => Html(html).into_response(),
        Err(e) => page(
            StatusCode::INTERNAL_SERVER_ERROR,
            "Not drawn",
            &format!("The list could not be drawn: {e}"),
        ),
    }
}

/// `GET /m/app.css`, from the exe.
async fn stylesheet() -> Response {
    (
        [(header::CONTENT_TYPE, "text/css; charset=utf-8")],
        crate::pages::STYLESHEET,
    )
        .into_response()
}

/// `GET /m/app.js` (EP-30b): the tool's own script, from the exe.
async fn script() -> Response {
    (
        [(header::CONTENT_TYPE, "text/javascript; charset=utf-8")],
        crate::pages::SCRIPT,
    )
        .into_response()
}

/// `GET /m/events[?since=<version>]` (EP-30b): the stream the pages' script listens to. A first
/// event sets the browser's retry (it carries no data, so no page hears it); then a `change` event
/// for each part that changed, its data the path of the page that shows it, or `*` first when the
/// window has moved on since the version the page was drawn from; a comment every 15 s keeps an
/// idle stream open through a proxy. It ends with the window's publisher, at the quit.
async fn events(State(shared): State<Arc<Shared>>, uri: Uri) -> Response {
    let Some(pages) = &shared.pages else {
        return not_served();
    };
    let since = uri.query().and_then(|query| {
        form::field(&form::decode(query.as_bytes()), "since").and_then(|v| v.parse().ok())
    });
    let first = futures_util::stream::once(async {
        Ok::<Event, Infallible>(
            Event::default()
                .retry(RETRY)
                .comment("the window's changes"),
        )
    });
    let changes = pages
        .published
        .changes(since)
        .map(|path| Ok(Event::default().event("change").data(path)));
    Sse::new(first.chain(changes))
        .keep_alive(KeepAlive::default())
        .into_response()
}

/// `GET /m/p/<project>/start` (EP-30b): the start's confirmation, from the window's preview,
/// asked through a channel of its own and waited for at most [`PREVIEW_WAIT`]; a project the
/// window does not list is a page not found.
async fn start_page(State(shared): State<Arc<Shared>>, Path(project): Path<String>) -> Response {
    let Some(pages) = &shared.pages else {
        return not_served();
    };
    // Read before the preview, so that a change made while it is drawn is caught up by the
    // event stream's `since`.
    let version = pages.published.version();
    let (reply, answer) = oneshot::channel();
    if shared.asks.send(StartAsk { project, reply }).is_err() {
        return page(
            StatusCode::SERVICE_UNAVAILABLE,
            "Not drawn",
            "The window is not listening.",
        );
    }
    (shared.wake)();
    match tokio::time::timeout(PREVIEW_WAIT, answer).await {
        Ok(Ok(Some(start))) => match crate::pages::start(&start, chrono::Utc::now(), version) {
            Ok(html) => Html(html).into_response(),
            Err(e) => page(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Not drawn",
                &format!("The start page could not be drawn: {e}"),
            ),
        },
        Ok(Ok(None)) => page(
            StatusCode::NOT_FOUND,
            "Not found",
            "No such project here: the project list is at /m/.",
        ),
        _ => page(
            StatusCode::SERVICE_UNAVAILABLE,
            "Not drawn",
            &format!(
                "The window did not answer within {} s: reload in a moment.",
                PREVIEW_WAIT.as_secs()
            ),
        ),
    }
}

/// `POST /m/p/<project>/start` (EP-30b): the next brief started headless with the rules' choice,
/// `ep` naming the brief the start page showed; back to the project's page, where the run shows.
async fn start(
    State(shared): State<Arc<Shared>>,
    Path(project): Path<String>,
    Extension(caller): Extension<Caller>,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    let back = project_path(&project);
    let Some(fields) = form_fields(&headers, &body) else {
        return not_a_form();
    };
    let ep = form::field(&fields, "ep").unwrap_or_default().trim();
    if ep.is_empty() {
        return shared.back(
            &back,
            Err("The form names no brief: nothing was started.".to_owned()),
        );
    }
    let order = Order::Start {
        project,
        ep: ep.to_owned(),
    };
    shared.order(order, &caller, &back).await
}

/// `POST /m/p/<project>/stop` (EP-30b): the window's run `key` of `ep` stopped gracefully.
async fn stop(
    State(shared): State<Arc<Shared>>,
    Path(project): Path<String>,
    Extension(caller): Extension<Caller>,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    let back = project_path(&project);
    let Some(fields) = form_fields(&headers, &body) else {
        return not_a_form();
    };
    let key = form::field(&fields, "key").and_then(|key| key.parse().ok());
    let ep = form::field(&fields, "ep").unwrap_or_default().trim();
    let Some(key) = key.filter(|_| !ep.is_empty()) else {
        return shared.back(
            &back,
            Err("The form names no run: nothing was stopped.".to_owned()),
        );
    };
    let order = Order::Stop {
        project,
        key,
        ep: ep.to_owned(),
    };
    shared.order(order, &caller, &back).await
}

/// `POST /m/autopilot` (EP-30b): every chain paused (`pause=on`) or resumed (`pause=off`); back
/// to the list, whose autopilot block shows it.
async fn pause(
    State(shared): State<Arc<Shared>>,
    Extension(caller): Extension<Caller>,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    let Some(fields) = form_fields(&headers, &body) else {
        return not_a_form();
    };
    let paused = match form::field(&fields, "pause") {
        Some("on") => true,
        Some("off") => false,
        _ => {
            return shared.back(
                PAGES_ROOT,
                Err("The form says neither pause nor resume: nothing was changed.".to_owned()),
            );
        }
    };
    shared
        .order(Order::Pause { paused }, &caller, PAGES_ROOT)
        .await
}

/// Any other `/m/…` path, once it passed the check.
async fn not_found() -> Response {
    page(
        StatusCode::NOT_FOUND,
        "Not found",
        "No page here: the project list is at /m/.",
    )
}

/// What a page says when the server holds no pages (the relays' tests).
fn not_served() -> Response {
    page(
        StatusCode::SERVICE_UNAVAILABLE,
        "Not served",
        "This server does not serve the pages.",
    )
}

/// `GET /m/p/<project>` (EP-30a): the project's page, with the outcome of the form that led here;
/// a project the window does not list is a page not found.
async fn project_page(
    State(shared): State<Arc<Shared>>,
    Path(project): Path<String>,
    uri: Uri,
) -> Response {
    let Some(pages) = &shared.pages else {
        return not_served();
    };
    let published = pages.published.fresh(|| (shared.wake)()).await;
    let said = shared.said(&uri);
    match crate::pages::project(&published, &project, said.as_ref()) {
        Some(Ok(html)) => Html(html).into_response(),
        Some(Err(e)) => page(
            StatusCode::INTERNAL_SERVER_ERROR,
            "Not drawn",
            &format!("The project's page could not be drawn: {e}"),
        ),
        // A form posted to a project the window does not list keeps its line (the review's).
        None => page(
            StatusCode::NOT_FOUND,
            "Not found",
            &match said {
                Some(Ok(words) | Err(words)) => {
                    format!("{words} No such project here: the project list is at /m/.")
                }
                None => "No such project here: the project list is at /m/.".to_owned(),
            },
        ),
    }
}

/// `GET /m/decisions` (EP-30a): what waits for the owner's answer, with the outcome of the form
/// that led here.
async fn decisions_page(State(shared): State<Arc<Shared>>, uri: Uri) -> Response {
    let Some(pages) = &shared.pages else {
        return not_served();
    };
    let published = pages.published.fresh(|| (shared.wake)()).await;
    let said = shared.said(&uri);
    match crate::pages::decisions(&published, said.as_ref()) {
        Ok(html) => Html(html).into_response(),
        Err(e) => page(
            StatusCode::INTERNAL_SERVER_ERROR,
            "Not drawn",
            &format!("The decisions could not be drawn: {e}"),
        ),
    }
}

/// `POST /m/p/<project>/tick`: an owner task ticked, by its id.
async fn tick_task(
    State(shared): State<Arc<Shared>>,
    Path(project): Path<String>,
    Extension(caller): Extension<Caller>,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    let back = project_path(&project);
    let Some(fields) = form_fields(&headers, &body) else {
        return not_a_form();
    };
    let id = form::field(&fields, "id").unwrap_or_default().trim();
    if id.is_empty() {
        return shared.back(
            &back,
            Err("No task was named: nothing was ticked.".to_owned()),
        );
    }
    let order = Order::TickTask {
        project,
        id: id.to_owned(),
    };
    shared.order(order, &caller, &back).await
}

/// `POST /m/p/<project>/message`: an inbox message ticked with the disposition typed for it,
/// which the window refuses when it is empty.
async fn tick_message(
    State(shared): State<Arc<Shared>>,
    Path(project): Path<String>,
    Extension(caller): Extension<Caller>,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    let back = project_path(&project);
    let Some(fields) = form_fields(&headers, &body) else {
        return not_a_form();
    };
    let id = form::field(&fields, "id").unwrap_or_default().trim();
    if id.is_empty() {
        return shared.back(
            &back,
            Err("No message was named: nothing was ticked.".to_owned()),
        );
    }
    let order = Order::TickMessage {
        project,
        id: id.to_owned(),
        disposition: form::field(&fields, "disposition")
            .unwrap_or_default()
            .trim()
            .to_owned(),
    };
    shared.order(order, &caller, &back).await
}

/// `POST /m/p/<project>/send`: the owner's message written into the project's inbox.
async fn send_message(
    State(shared): State<Arc<Shared>>,
    Path(project): Path<String>,
    Extension(caller): Extension<Caller>,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    let back = project_path(&project);
    let Some(fields) = form_fields(&headers, &body) else {
        return not_a_form();
    };
    let order = Order::Send {
        project,
        text: form::field(&fields, "text").unwrap_or_default().to_owned(),
        urgent: form::field(&fields, "urgent").is_some(),
    };
    shared.order(order, &caller, &back).await
}

/// `POST /m/decisions/answer`: a waiting request answered, or a deferred run's.
async fn answer(
    State(shared): State<Arc<Shared>>,
    Extension(caller): Extension<Caller>,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    let Some(fields) = form_fields(&headers, &body) else {
        return not_a_form();
    };
    match answer_order(&fields) {
        Ok(order) => shared.order(order, &caller, DECISIONS_ROUTE).await,
        Err(why) => shared.back(DECISIONS_ROUTE, Err(why)),
    }
}

/// An answer form's fields as an order: `key`, `request`, `do`, `reason`, `pick<q>` (once for
/// each option picked) and `other<q>`.
fn answer_order(fields: &[(String, String)]) -> Result<Order, String> {
    let key = form::field(fields, "key")
        .and_then(|key| key.parse().ok())
        .ok_or("The form names no session: nothing was sent.")?;
    let request = form::field(fields, "request")
        .filter(|id| !id.is_empty())
        .ok_or("The form names no request: nothing was sent.")?;
    let verb = form::field(fields, "do")
        .and_then(Verb::parse)
        .ok_or("The form names no answer: nothing was sent.")?;
    let mut form = AnswerForm {
        verb,
        picks: BTreeMap::new(),
        other: BTreeMap::new(),
        reason: form::field(fields, "reason").unwrap_or_default().to_owned(),
    };
    for (name, value) in fields {
        if let Some(q) = name.strip_prefix("pick").and_then(|q| q.parse().ok()) {
            if let Ok(option) = value.parse() {
                form.picks.entry(q).or_default().insert(option);
            }
        } else if let Some(q) = name.strip_prefix("other").and_then(|q| q.parse().ok()) {
            form.other.insert(q, value.clone());
        }
    }
    Ok(Order::Answer {
        key,
        request: request.to_owned(),
        form,
    })
}

/// A form's fields, once its body is a form's; none for a body of another kind.
fn form_fields(headers: &HeaderMap, body: &Bytes) -> Option<Vec<(String, String)>> {
    let kind = headers
        .get(header::CONTENT_TYPE)
        .and_then(|kind| kind.to_str().ok())
        .unwrap_or_default();
    form::is_form(kind).then(|| form::decode(body))
}

/// What a body that is not a form's gets.
fn not_a_form() -> Response {
    page(
        StatusCode::UNSUPPORTED_MEDIA_TYPE,
        "Not a form",
        "This address takes a page's form only.",
    )
}

/// The path of the project's page, which the event stream names for its part too (EP-30b).
pub fn project_path(project: &str) -> String {
    format!("{PROJECT_PREFIX}{}", form::segment(project))
}

/// A second window's request that this one show itself (EP-40): handed to the window, which
/// takes it as its tray's Show.
async fn show(State(shared): State<Arc<Shared>>, headers: HeaderMap) -> StatusCode {
    shared.hand(&headers, Signal::Show)
}

/// `crw install`'s request (EP-37): handed to the window, whose answer goes back; none within
/// [`RESTART_WAIT`] is a 504.
async fn restart(
    State(shared): State<Arc<Shared>>,
    headers: HeaderMap,
    Json(ask): Json<RestartAsk>,
) -> Result<Json<RestartAnswer>, StatusCode> {
    if !shared.local(&headers) {
        return Err(StatusCode::FORBIDDEN);
    }
    let (reply, answer) = oneshot::channel();
    shared
        .restarts
        .send(RestartRequest {
            from: ask.from,
            bin: ask.bin,
            reply,
        })
        .map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
    (shared.wake)();
    match tokio::time::timeout(RESTART_WAIT, answer).await {
        Ok(Ok(answer)) => Ok(Json(answer)),
        _ => Err(StatusCode::GATEWAY_TIMEOUT),
    }
}

async fn hook(
    State(shared): State<Arc<Shared>>,
    headers: HeaderMap,
    Json(hook): Json<Hook>,
) -> StatusCode {
    shared.hand(&headers, Signal::Hook(hook))
}

async fn status_line(
    State(shared): State<Arc<Shared>>,
    headers: HeaderMap,
    Json(line): Json<StatusLine>,
) -> StatusCode {
    shared.hand(&headers, Signal::StatusLine(Box::new(line)))
}

#[derive(Serialize)]
struct Health {
    app: &'static str,
    version: &'static str,
}

async fn health(
    State(shared): State<Arc<Shared>>,
    headers: HeaderMap,
) -> Result<Json<Health>, StatusCode> {
    if !shared.local(&headers) {
        return Err(StatusCode::FORBIDDEN);
    }
    Ok(Json(Health {
        app: signals::APP_NAME,
        version: env!("CARGO_PKG_VERSION"),
    }))
}

impl Shared {
    /// A request from the loopback that no web page made: no `Origin`, and the loopback in `Host`.
    fn local(&self, headers: &HeaderMap) -> bool {
        if headers.contains_key(header::ORIGIN) {
            return false;
        }
        headers
            .get(header::HOST)
            .and_then(|host| host.to_str().ok())
            .is_some_and(|host| remote::is_loopback(host, self.port))
    }

    /// Writes a line of the remote-action log (EP-30) off the server's thread and waits for it:
    /// `false` when it was not written, or no ledger is set.
    async fn log(&self, detail: String) -> bool {
        let Some(ledger) = self.pages.as_ref().and_then(|pages| pages.ledger.clone()) else {
            return false;
        };
        let at = chrono::Utc::now();
        tokio::task::spawn_blocking(move || {
            Ledger::open(&ledger)
                .and_then(|ledger| ledger.add_event(at, remote::EVENT_KIND, None, &detail))
                .is_ok()
        })
        .await
        .unwrap_or(false)
    }

    /// The outcome a redirect's `done=<n>` names, while it is kept (EP-30a).
    fn said(&self, uri: &Uri) -> Option<Said> {
        let fields = form::decode(uri.query()?.as_bytes());
        let number = form::field(&fields, "done")?.parse().ok()?;
        self.outcomes.lock().ok()?.get(number, uri.path())
    }

    /// Back to the page at `path` with `said` kept for it: a 303, so a reload never posts again.
    fn back(&self, path: &str, said: Said) -> Response {
        match self.outcomes.lock() {
            Ok(mut outcomes) => {
                let number = outcomes.put(path, said);
                Redirect::to(&format!("{path}?done={number}")).into_response()
            }
            Err(_) => Redirect::to(path).into_response(),
        }
    }

    /// Hands `order` to the window and waits at most [`FORM_WAIT`] for what came of it, then
    /// back to the page at `path` (EP-30a). A form the window has not taken by then is let go
    /// and never run; one it took is said to be under way.
    async fn order(&self, order: Order, caller: &Caller, path: &str) -> Response {
        let (reply, outcome) = oneshot::channel();
        let hand = Hand::default();
        let request = PageRequest {
            order,
            caller: caller.identity().to_owned(),
            reply,
            hand: hand.clone(),
        };
        let before = self.pages.as_ref().map(|pages| pages.published.version());
        if self.orders.send(request).is_err() {
            return self.back(
                path,
                Err("The window is not listening: nothing was done.".to_owned()),
            );
        }
        (self.wake)();
        let said = match tokio::time::timeout(FORM_WAIT, outcome).await {
            Ok(Ok(said)) => said,
            Ok(Err(_)) => Err("The window let the form go: nothing was done.".to_owned()),
            Err(_) if hand.let_go() => Err(format!(
                "The window did not take the form within {} s, so nothing was done: send it \
                 again when the window answers.",
                FORM_WAIT.as_secs()
            )),
            Err(_) => Err(format!(
                "The window took the form and is still at it after {} s: look at the page in a \
                 moment rather than send it again.",
                FORM_WAIT.as_secs()
            )),
        };
        // What it changed shows on the page the redirect brings, once the window has read the
        // files again and published (a moment at most).
        if said.is_ok()
            && let (Some(pages), Some(before)) = (&self.pages, before)
        {
            pages.published.newer_than(before, SHOWN_WAIT).await;
        }
        self.back(path, said)
    }

    /// The same without waiting: a refusal's line, which nothing waits on.
    fn log_detached(&self, detail: String) {
        let Some(ledger) = self.pages.as_ref().and_then(|pages| pages.ledger.clone()) else {
            return;
        };
        let at = chrono::Utc::now();
        drop(tokio::task::spawn_blocking(move || {
            let _ = Ledger::open(&ledger)
                .and_then(|ledger| ledger.add_event(at, remote::EVENT_KIND, None, &detail));
        }));
    }

    fn hand(&self, headers: &HeaderMap, signal: Signal) -> StatusCode {
        if !self.local(headers) {
            return StatusCode::FORBIDDEN;
        }
        if self.sender.send(signal).is_err() {
            return StatusCode::SERVICE_UNAVAILABLE;
        }
        (self.wake)();
        StatusCode::NO_CONTENT
    }
}

#[cfg(test)]
mod tests {
    use std::io::{Read, Write};
    use std::net::TcpStream;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::time::{Duration, Instant};

    use crw_core::signals::relay;

    use super::*;

    /// A server on a free port, once it listens.
    fn listening() -> (Server, u16, Arc<AtomicUsize>) {
        let woken = Arc::new(AtomicUsize::new(0));
        let counter = Arc::clone(&woken);
        let mut server = Server::start(0, move || {
            counter.fetch_add(1, Ordering::SeqCst);
        });
        let deadline = Instant::now() + Duration::from_secs(10);
        loop {
            server.receive();
            match &server.listening {
                Listening::On(port) => {
                    let port = *port;
                    return (server, port, woken);
                }
                Listening::Taken(_, why) | Listening::Failed(why) => panic!("{why}"),
                Listening::Starting(_) if Instant::now() > deadline => panic!("never listened"),
                Listening::Starting(_) => std::thread::sleep(Duration::from_millis(10)),
            }
        }
    }

    /// The status code of a request written by hand.
    fn raw(port: u16, request: &str) -> u16 {
        let mut stream = TcpStream::connect(("127.0.0.1", port)).unwrap();
        stream
            .set_read_timeout(Some(Duration::from_secs(5)))
            .unwrap();
        stream.write_all(request.as_bytes()).unwrap();
        let mut answer = String::new();
        stream.read_to_string(&mut answer).unwrap();
        answer
            .split_whitespace()
            .nth(1)
            .and_then(|code| code.parse().ok())
            .unwrap_or_default()
    }

    /// The signals heard within a few seconds.
    fn heard(server: &mut Server, wanted: usize) -> Vec<Signal> {
        let deadline = Instant::now() + Duration::from_secs(5);
        let mut all = Vec::new();
        while all.len() < wanted && Instant::now() < deadline {
            all.extend(server.receive());
            std::thread::sleep(Duration::from_millis(10));
        }
        all
    }

    #[test]
    fn the_relays_reach_the_window() {
        let (mut server, port, woken) = listening();
        let input = r#"{"hook_event_name":"SessionStart","session_id":"s1","cwd":"C:\\Data\\alpha","source":"startup","transcript_path":"x"}"#;
        let mut out = Vec::new();
        signals::relay_hook(
            input.as_bytes(),
            Some("claude-vscode".to_owned()),
            false,
            Vec::new,
            &mut out,
            port,
        )
        .unwrap();
        signals::relay_status_line(
            r#"{"session_id":"s1","workspace":{"project_dir":"C:\\Data\\alpha"},"cost":{"total_cost_usd":0.5}}"#
                .as_bytes(),
            None,
            port,
        )
        .unwrap();
        let all = heard(&mut server, 2);
        assert_eq!(all.len(), 2, "{all:?}");
        let Signal::Hook(hook) = &all[0] else {
            panic!("{all:?}")
        };
        assert_eq!(
            (hook.event.as_str(), hook.session_id.as_str()),
            ("SessionStart", "s1")
        );
        assert_eq!(hook.entrypoint.as_deref(), Some("claude-vscode"));
        let Signal::StatusLine(line) = &all[1] else {
            panic!("{all:?}")
        };
        assert_eq!(line.cost.total_cost_usd, Some(0.5));
        assert!(
            woken.load(Ordering::SeqCst) >= 3,
            "woken at the bind and at each signal"
        );
        // The health of the app, for `crw signals` and EP-28's single instance.
        let answer = relay::request(
            port,
            "GET",
            signals::HEALTH_ROUTE,
            None,
            Duration::from_secs(2),
        )
        .unwrap();
        assert_eq!(answer.status, 200);
        assert!(
            String::from_utf8(answer.body)
                .unwrap()
                .contains("clauderoadmapwrangler")
        );
    }

    #[test]
    fn only_the_loopback_without_a_page_gets_through() {
        let (mut server, port, _) = listening();
        let body = r#"{"hook_event_name":"Stop","session_id":"s1"}"#;
        let post = |extra: &str, host: &str, kind: &str| {
            format!(
                "POST /hook HTTP/1.1\r\nHost: {host}\r\n{extra}Content-Type: {kind}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                body.len()
            )
        };
        let local = format!("127.0.0.1:{port}");
        assert_eq!(raw(port, &post("", &local, "application/json")), 204);
        assert_eq!(
            raw(
                port,
                &post("", &format!("localhost:{port}"), "application/json")
            ),
            204
        );
        // A web page's request, a name pointed at the loopback, a body that is not JSON.
        assert_eq!(
            raw(
                port,
                &post(
                    "Origin: http://example.test\r\n",
                    &local,
                    "application/json"
                )
            ),
            403
        );
        assert_eq!(
            raw(
                port,
                &post("", &format!("example.test:{port}"), "application/json")
            ),
            403
        );
        assert_eq!(raw(port, &post("", &local, "text/plain")), 415);
        let get_health = |extra: &str| {
            format!("GET /health HTTP/1.1\r\nHost: {local}\r\n{extra}Connection: close\r\n\r\n")
        };
        assert_eq!(raw(port, &get_health("")), 200);
        assert_eq!(
            raw(port, &get_health("Origin: http://example.test\r\n")),
            403
        );
        assert_eq!(heard(&mut server, 2).len(), 2, "the two local posts only");
        // Another server on the same port is told the port is taken (EP-40's guard asks whose).
        let mut second = Server::start(port, || {});
        let deadline = Instant::now() + Duration::from_secs(5);
        while matches!(second.listening, Listening::Starting(_)) && Instant::now() < deadline {
            second.receive();
            std::thread::sleep(Duration::from_millis(10));
        }
        assert!(
            matches!(&second.listening, Listening::Taken(taken, why) if *taken == port && why.contains(&local)),
            "{:?}",
            second.listening
        );
    }

    /// A second window's yield (EP-40): `/health` names the app, `POST /show` reaches the window
    /// as a signal, and a page's request is refused.
    #[test]
    fn a_second_window_asks_the_first_to_show_itself() {
        let (mut server, port, _) = listening();
        // A start at login only asks whether a window is there (EP-28a): nothing is shown.
        assert_eq!(signals::ask_show(port, false), signals::Yield::Present);
        assert_eq!(signals::ask_show(port, true), signals::Yield::Shown);
        assert_eq!(heard(&mut server, 1), vec![Signal::Show]);
        let local = format!("127.0.0.1:{port}");
        assert_eq!(
            raw(
                port,
                &format!(
                    "POST /show HTTP/1.1\r\nHost: {local}\r\nOrigin: http://example.test\r\n\
                     Content-Length: 0\r\nConnection: close\r\n\r\n"
                )
            ),
            403
        );
        // A port nothing listens on, or something that is not a window, is no window.
        drop(server);
        let free = {
            let probe = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
            probe.local_addr().unwrap().port()
        };
        assert_eq!(signals::ask_show(free, true), signals::Yield::NoWindow);
    }

    /// `crw install`'s request (EP-37) reaches the window, and the window's answer goes back; a
    /// page's request is refused before the window hears of it.
    #[test]
    fn a_restart_request_waits_for_the_windows_answer() {
        let (mut server, port, _) = listening();
        let client = std::thread::spawn(move || {
            signals::ask_restart(
                port,
                "a test",
                Some(std::path::Path::new(r"C:\scratch\bin")),
            )
        });
        let deadline = Instant::now() + Duration::from_secs(5);
        let request = loop {
            if let Some(request) = server.take_restarts().pop() {
                break request;
            }
            assert!(Instant::now() < deadline, "the request never came");
            std::thread::sleep(Duration::from_millis(10));
        };
        assert_eq!(request.from, "a test");
        assert_eq!(
            request.bin.as_deref(),
            Some(std::path::Path::new(r"C:\scratch\bin"))
        );
        let answer = RestartAnswer {
            restarting: false,
            words: "a headless run works".to_owned(),
        };
        request.reply.send(answer.clone()).unwrap();
        assert_eq!(
            client.join().unwrap(),
            signals::RestartReply::Answered(answer)
        );
        let local = format!("127.0.0.1:{port}");
        let body = r#"{"from":"a page"}"#;
        let page = format!(
            "POST /restart HTTP/1.1\r\nHost: {local}\r\nOrigin: http://example.test\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
            body.len()
        );
        assert_eq!(raw(port, &page), 403);
        assert!(server.take_restarts().is_empty());
    }

    /// An answer read whole: its status, its head in lower case and its body.
    struct Answer {
        status: u16,
        head: String,
        body: String,
    }

    /// A request written by hand with the caller's own headers, its answer read whole.
    fn exchange(port: u16, method: &str, path: &str, headers: &[(&str, &str)]) -> Answer {
        let mut request = format!("{method} {path} HTTP/1.1\r\n");
        for (name, value) in headers {
            request.push_str(&format!("{name}: {value}\r\n"));
        }
        request.push_str("Content-Length: 0\r\nConnection: close\r\n\r\n");
        let mut stream = TcpStream::connect(("127.0.0.1", port)).unwrap();
        stream
            .set_read_timeout(Some(Duration::from_secs(10)))
            .unwrap();
        stream.write_all(request.as_bytes()).unwrap();
        let mut bytes = Vec::new();
        stream.read_to_end(&mut bytes).unwrap();
        let text = String::from_utf8_lossy(&bytes).into_owned();
        let (head, body) = text.split_once("\r\n\r\n").unwrap_or((&text, ""));
        Answer {
            status: head
                .split_whitespace()
                .nth(1)
                .and_then(|code| code.parse().ok())
                .unwrap_or_default(),
            head: head.to_ascii_lowercase(),
            body: body.to_owned(),
        }
    }

    /// The remote-action log's lines, once `wanted` are written (a refusal's is written off the
    /// server's thread), then a moment more for any line too many.
    fn remote_lines(ledger: &std::path::Path, wanted: usize) -> Vec<String> {
        let read = || {
            Ledger::open(ledger)
                .and_then(|ledger| ledger.events(remote::EVENT_KIND, None))
                .map(|rows| rows.into_iter().filter_map(|row| row.detail).collect())
                .unwrap_or_default()
        };
        let deadline = Instant::now() + Duration::from_secs(5);
        let mut lines: Vec<String> = read();
        while lines.len() < wanted && Instant::now() < deadline {
            std::thread::sleep(Duration::from_millis(20));
            lines = read();
        }
        std::thread::sleep(Duration::from_millis(200));
        read()
    }

    /// A server with the pages: a list of one fake project published now, a scratch registry
    /// naming `owner` when given, a scratch ledger. The publisher is kept, as the window keeps it.
    fn serving(
        scratch: &crate::portfolio::tests::Scratch,
        owner: Option<&str>,
    ) -> (Server, u16, PathBuf, crate::published::Publisher) {
        use crate::published::{Group, Line, List, Next, Publisher};
        use crw_core::registry::{Defaults, State};

        let registry = scratch.0.join("registry.toml");
        let mut text = "format = 1\n".to_owned();
        if let Some(owner) = owner {
            text.push_str(&format!("\n[settings]\nowner_login = \"{owner}\"\n"));
        }
        std::fs::write(&registry, text).unwrap();
        let store = Store::new(
            registry,
            scratch.0.join("backups"),
            Defaults {
                repository: scratch.0.join("harbor"),
                data_root: scratch.0.clone(),
            },
        );
        let ledger = scratch.0.join("ledger.sqlite");
        let mut publisher = Publisher::default();
        let harbor = Line {
            name: "harbor".to_owned(),
            state: State::Active,
            pinned: false,
            proposes: false,
            autopilot: false,
            next: Next::Brief {
                id: "EP-4".to_owned(),
                size: Some("M".to_owned()),
                title: "The `scan` view".to_owned(),
            },
            counts: Default::default(),
            owner_items: 1,
            messages: 0,
            urgent: 0,
            sessions: Vec::new(),
            usage_tenths: None,
            badges: Vec::new(),
        };
        let page = crate::published::Project {
            name: "harbor".to_owned(),
            state: State::Active,
            folder: true,
            next: harbor.next.clone(),
            note: Some("The view's rows. **From EP-3:** (1) the table; (2) its sort.".to_owned()),
            waiting: vec![crate::published::Item {
                id: Some("OT-3".to_owned()),
                gate: None,
                text: "Look at the list.".to_owned(),
                meta: "roadmap/owner-tasks.md:5".to_owned(),
                tickable: true,
            }],
            resolved: 0,
            uncounted: Vec::new(),
            inbox: None,
            sessions: Vec::new(),
            busy: None,
            revert: false,
            pinned: false,
            proposes: false,
            autopilot: false,
            runs: Vec::new(),
        };
        publisher.offer(
            crate::published::Parts {
                list: List {
                    groups: vec![Group {
                        state: State::Active,
                        projects: vec![harbor],
                    }],
                    ..List::default()
                },
                projects: vec![page],
                decisions: crate::published::Decisions::default(),
            },
            chrono::Utc::now(),
        );
        let pages = Pages {
            published: publisher.reader(),
            store: Arc::new(store),
            ledger: Some(ledger.clone()),
        };
        let mut server = Server::serve(0, Some(pages), || {});
        let deadline = Instant::now() + Duration::from_secs(10);
        loop {
            server.receive();
            match &server.listening {
                Listening::On(port) => {
                    let port = *port;
                    return (server, port, ledger, publisher);
                }
                Listening::Taken(_, why) | Listening::Failed(why) => panic!("{why}"),
                Listening::Starting(_) if Instant::now() > deadline => panic!("never listened"),
                Listening::Starting(_) => std::thread::sleep(Duration::from_millis(10)),
            }
        }
    }

    /// The machine itself reads the list, its stylesheet and the redirect to it, each page with
    /// its headers; a path with no page is a page not found.
    #[test]
    fn the_machine_itself_reads_the_list() {
        let scratch = crate::portfolio::tests::Scratch::new("server-pages-local");
        let (_server, port, ledger, _publisher) = serving(&scratch, None);
        let local = format!("127.0.0.1:{port}");
        let list = exchange(port, "GET", "/m/", &[("Host", &local)]);
        assert_eq!(list.status, 200, "{}", list.head);
        assert!(
            list.head.contains("content-type: text/html"),
            "{}",
            list.head
        );
        assert!(list.body.contains("harbor"), "{}", list.body);
        assert!(list.body.contains("<code>scan</code>"), "{}", list.body);
        assert!(list.body.contains("1 owner item"), "{}", list.body);
        for header in [
            "cache-control: no-store",
            "x-content-type-options: nosniff",
            "referrer-policy: same-origin",
            "x-frame-options: deny",
            "content-security-policy: default-src 'none'; style-src 'self'",
        ] {
            assert!(list.head.contains(header), "{header}: {}", list.head);
        }
        let bare = exchange(port, "GET", "/m", &[("Host", &local)]);
        assert_eq!(bare.status, 303);
        assert!(bare.head.contains("location: /m/\r\n") || bare.head.ends_with("location: /m/"));
        let localhost = format!("localhost:{port}");
        let css = exchange(port, "GET", "/m/app.css", &[("Host", &localhost)]);
        assert_eq!(css.status, 200);
        assert!(css.head.contains("content-type: text/css"), "{}", css.head);
        assert!(css.body.contains("prefers-color-scheme"));
        let none = exchange(port, "GET", "/m/nothing/here", &[("Host", &local)]);
        assert_eq!(none.status, 404);
        assert!(none.body.contains("Not found"));
        // A form from the page itself: no such route yet, and nothing logged for the machine.
        let origin = format!("http://{local}");
        let form = exchange(
            port,
            "POST",
            "/m/",
            &[("Host", &local), ("Origin", &origin)],
        );
        assert_eq!(form.status, 405);
        assert!(
            remote_lines(&ledger, 0).is_empty(),
            "the machine's own requests are not logged"
        );
    }

    /// Through Serve only the owner's login passes. A stranger, a device with no user, a page of
    /// another origin, a page that reached the loopback through a name of its own (DNS rebinding)
    /// and a bare foreign host are refused with a page that names no reason, each identity logged
    /// once within the minute; the owner's form is logged before it runs.
    #[test]
    fn only_the_owner_passes_through_serve_and_each_refusal_is_logged() {
        let scratch = crate::portfolio::tests::Scratch::new("server-pages-serve");
        let (_server, port, ledger, _publisher) = serving(&scratch, Some("owner@example.com"));
        let through = |login: &str| {
            vec![
                ("Host", "box.example.ts.net:9443".to_owned()),
                ("X-Forwarded-Host", "box.example.ts.net:9443".to_owned()),
                ("X-Forwarded-For", "100.64.0.2".to_owned()),
                ("X-Forwarded-Proto", "https".to_owned()),
                ("Tailscale-User-Login", login.to_owned()),
            ]
        };
        let send = |method: &str, headers: &[(&str, String)]| {
            let headers: Vec<(&str, &str)> = headers
                .iter()
                .map(|(name, value)| (*name, value.as_str()))
                .collect();
            exchange(port, method, "/m/", &headers)
        };
        let owner = send("GET", &through("Owner@Example.com"));
        assert_eq!(owner.status, 200, "the owner, case aside");
        assert!(owner.body.contains("harbor"));
        let stranger = send("GET", &through("someone@example.org"));
        assert_eq!(stranger.status, 403);
        assert!(stranger.body.contains("answers its owner only"));
        assert!(!stranger.body.contains("harbor"), "nothing of the list");
        assert!(stranger.head.contains("cache-control: no-store"));
        // The same stranger again within the minute: refused, not logged again.
        assert_eq!(send("GET", &through("someone@example.org")).status, 403);
        // A device with no user, whichever Host Serve passes.
        let mut anonymous = through("");
        anonymous.retain(|(name, _)| *name != "Tailscale-User-Login");
        assert_eq!(send("GET", &anonymous).status, 403);
        anonymous[0].1 = format!("127.0.0.1:{port}");
        assert_eq!(send("GET", &anonymous).status, 403);
        // A page of another origin in the owner's own browser.
        let mut foreign = through("owner@example.com");
        foreign.push(("Origin", "https://evil.example".to_owned()));
        assert_eq!(send("GET", &foreign).status, 403);
        // A page that reached the loopback through a name of its own, sending the owner's login
        // and forwarding headers of its choosing: its Host gives it away (the review's finding).
        let rebinding = vec![
            ("Host", format!("attacker.example:{port}")),
            ("Tailscale-User-Login", "owner@example.com".to_owned()),
        ];
        assert_eq!(send("GET", &rebinding).status, 403);
        let mut forged = rebinding.clone();
        forged.extend([
            ("Origin", format!("http://attacker.example:{port}")),
            ("X-Forwarded-Proto", "http".to_owned()),
            ("X-Forwarded-Host", format!("attacker.example:{port}")),
        ]);
        assert_eq!(send("POST", &forged).status, 403);
        // A bare foreign Host, no login and no proxy header.
        let bare = [("Host", format!("attacker.example:{port}"))];
        assert_eq!(send("GET", &bare).status, 403);
        // The owner's form, naming the page's own origin: logged, then no such route yet.
        let mut form = through("owner@example.com");
        form.push(("Origin", "https://box.example.ts.net:9443".to_owned()));
        assert_eq!(send("POST", &form).status, 405);
        let lines = remote_lines(&ledger, 4);
        assert_eq!(
            lines
                .iter()
                .filter(|line| line.ends_with("· allowed"))
                .collect::<Vec<_>>(),
            ["owner@example.com · POST /m/ · allowed"],
            "the forged form never passed: {lines:#?}"
        );
        assert_eq!(
            lines
                .iter()
                .filter(|line| line.starts_with("someone@example.org · GET /m/ · refused"))
                .count(),
            1,
            "{lines:#?}"
        );
        assert_eq!(
            lines
                .iter()
                .filter(|line| line.starts_with("no login · GET /m/ · refused"))
                .count(),
            1,
            "the two anonymous requests within the minute: {lines:#?}"
        );
        assert!(
            lines.contains(&"owner@example.com · POST /m/ · allowed".to_owned()),
            "{lines:#?}"
        );
        assert!(
            lines.iter().all(|line| !line.contains("box.example")
                && !line.contains("100.64")
                && !line.contains("attacker")),
            "no header's value beyond the login: {lines:#?}"
        );
    }

    /// A form posted by hand with the caller's own headers: its answer read whole.
    fn post_form(port: u16, path: &str, headers: &[(&str, &str)], body: &str) -> Answer {
        let mut request = format!("POST {path} HTTP/1.1\r\n");
        for (name, value) in headers {
            request.push_str(&format!("{name}: {value}\r\n"));
        }
        request.push_str(&format!(
            "Content-Length: {}\r\nConnection: close\r\n\r\n{body}",
            body.len()
        ));
        let mut stream = TcpStream::connect(("127.0.0.1", port)).unwrap();
        stream
            .set_read_timeout(Some(Duration::from_secs(15)))
            .unwrap();
        stream.write_all(request.as_bytes()).unwrap();
        let mut bytes = Vec::new();
        stream.read_to_end(&mut bytes).unwrap();
        let text = String::from_utf8_lossy(&bytes).into_owned();
        let (head, body) = text.split_once("\r\n\r\n").unwrap_or((&text, ""));
        Answer {
            status: head
                .split_whitespace()
                .nth(1)
                .and_then(|code| code.parse().ok())
                .unwrap_or_default(),
            head: head.to_ascii_lowercase(),
            body: body.to_owned(),
        }
    }

    /// The `location` a redirect names.
    fn location(answer: &Answer) -> String {
        answer
            .head
            .lines()
            .find_map(|line| line.strip_prefix("location: "))
            .unwrap_or_default()
            .trim()
            .to_owned()
    }

    /// The orders the server hands the window within a few seconds, each answered with
    /// `answer` as the window would.
    fn obey(server: &mut Server, wanted: usize, answer: impl Fn(&Order) -> Said) -> Vec<Order> {
        let deadline = Instant::now() + Duration::from_secs(8);
        let mut orders = Vec::new();
        while orders.len() < wanted && Instant::now() < deadline {
            for request in server.take_orders() {
                let said = answer(&request.order);
                orders.push((request.order.clone(), request.caller.clone()));
                let _ = request.reply.send(said);
            }
            std::thread::sleep(Duration::from_millis(10));
        }
        orders
            .into_iter()
            .map(|(order, caller)| {
                assert!(!caller.is_empty());
                order
            })
            .collect()
    }

    /// The machine itself reads a project's page and the decisions; a project the window does
    /// not list is a page not found.
    #[test]
    fn the_machine_itself_reads_a_project_and_the_decisions() {
        let scratch = crate::portfolio::tests::Scratch::new("server-pages-project");
        let (_server, port, _ledger, _publisher) = serving(&scratch, None);
        let local = format!("127.0.0.1:{port}");
        let page = exchange(port, "GET", "/m/p/harbor", &[("Host", &local)]);
        assert_eq!(page.status, 200, "{}", page.body);
        assert!(
            page.body.contains(r#"<p class="label">From EP-3:</p>"#),
            "{}",
            page.body
        );
        assert!(page.body.contains(r#"action="/m/p/harbor/tick""#));
        assert!(
            page.head
                .contains("content-security-policy: default-src 'none'")
        );
        // The name's case aside, as the window finds a project.
        assert_eq!(
            exchange(port, "GET", "/m/p/Harbor", &[("Host", &local)]).status,
            200
        );
        let none = exchange(port, "GET", "/m/p/nowhere", &[("Host", &local)]);
        assert_eq!(none.status, 404);
        assert!(none.body.contains("No such project here"));
        let decisions = exchange(port, "GET", "/m/decisions", &[("Host", &local)]);
        assert_eq!(decisions.status, 200);
        assert!(decisions.body.contains("Nothing waits for you."));
        // A read of a form's path is no page.
        assert_eq!(
            exchange(port, "GET", "/m/p/harbor/tick", &[("Host", &local)]).status,
            405
        );
    }

    /// A form reaches the window as an order, its answer comes back as a redirect to its page
    /// with `done=<n>`, and the page then draws the line; a form without its origin, or with a
    /// body that is not a form, never reaches the window.
    #[test]
    fn a_form_reaches_the_window_and_its_page_says_what_came_of_it() {
        let scratch = crate::portfolio::tests::Scratch::new("server-pages-forms");
        let (mut server, port, ledger, mut publisher) = serving(&scratch, None);
        let local = format!("127.0.0.1:{port}");
        let origin = format!("http://{local}");
        let form = [
            ("Host", local.as_str()),
            ("Origin", origin.as_str()),
            ("Content-Type", "application/x-www-form-urlencoded"),
        ];
        let tick = {
            let form: Vec<(String, String)> = form
                .iter()
                .map(|(n, v)| ((*n).to_owned(), (*v).to_owned()))
                .collect();
            std::thread::spawn(move || {
                let form: Vec<(&str, &str)> =
                    form.iter().map(|(n, v)| (n.as_str(), v.as_str())).collect();
                post_form(port, "/m/p/harbor/tick", &form, "id=OT-3")
            })
        };
        let orders = obey(&mut server, 1, |_| Ok("Ticked OT-3 in harbor.".to_owned()));
        assert_eq!(
            orders,
            [Order::TickTask {
                project: "harbor".to_owned(),
                id: "OT-3".to_owned()
            }]
        );
        // The window reads the files again and publishes: the redirect waits for that.
        let current = publisher.current();
        let mut parts = crate::published::Parts {
            list: current.list.clone(),
            projects: current.projects.clone(),
            decisions: current.decisions.clone(),
        };
        parts.projects[0].waiting.clear();
        parts.projects[0].resolved = 1;
        publisher.offer(parts, chrono::Utc::now());
        let answer = tick.join().unwrap();
        assert_eq!(answer.status, 303, "{}", answer.head);
        let back = location(&answer);
        assert!(back.starts_with("/m/p/harbor?done="), "{back}");
        let page = exchange(port, "GET", &back, &[("Host", &local)]);
        assert!(
            page.body
                .contains(r#"<p class="said done">Ticked OT-3 in harbor.</p>"#),
            "{}",
            page.body
        );
        assert!(
            !page.body.contains("Tick OT-3") && page.body.contains("0 open, 1 resolved"),
            "the page shows the tick: {}",
            page.body
        );
        // A number the server never gave draws no line.
        let page = exchange(port, "GET", "/m/p/harbor?done=999", &[("Host", &local)]);
        assert!(!page.body.contains("class=\"said"));
        // The answer form, with its picks and words, and the window's refusal said.
        let answering = {
            let origin = origin.clone();
            let local = local.clone();
            std::thread::spawn(move || {
                post_form(
                    port,
                    "/m/decisions/answer",
                    &[
                        ("Host", &local),
                        ("Origin", &origin),
                        (
                            "Content-Type",
                            "application/x-www-form-urlencoded; charset=UTF-8",
                        ),
                    ],
                    "key=7&request=req-1&pick0=1&pick1=0&pick1=2&other1=more+rows&reason=&do=answer",
                )
            })
        };
        let orders = obey(&mut server, 1, |_| Err("Already answered.".to_owned()));
        let Order::Answer { key, request, form } = &orders[0] else {
            panic!("{orders:?}")
        };
        assert_eq!((*key, request.as_str()), (7, "req-1"));
        assert_eq!(form.verb, Verb::Answer);
        assert_eq!(form.picks[&0], BTreeSet::from([1]));
        assert_eq!(form.picks[&1], BTreeSet::from([0, 2]));
        assert_eq!(form.other[&1], "more rows");
        let answer = answering.join().unwrap();
        let back = location(&answer);
        assert!(back.starts_with("/m/decisions?done="), "{back}");
        let page = exchange(port, "GET", &back, &[("Host", &local)]);
        assert!(
            page.body
                .contains(r#"<p class="said failed">Already answered.</p>"#)
        );
        // A form with no answer named never reaches the window.
        let answer = post_form(
            port,
            "/m/decisions/answer",
            &form_headers(&local, &origin),
            "key=7",
        );
        assert_eq!(answer.status, 303);
        // Without its origin, or with a body that is not a form: refused before the window.
        let bare = post_form(
            port,
            "/m/p/harbor/tick",
            &[
                ("Host", &local),
                ("Content-Type", "application/x-www-form-urlencoded"),
            ],
            "id=OT-3",
        );
        assert_eq!(bare.status, 403);
        let json = post_form(
            port,
            "/m/p/harbor/tick",
            &[
                ("Host", &local),
                ("Origin", &origin),
                ("Content-Type", "application/json"),
            ],
            "{\"id\":\"OT-3\"}",
        );
        assert_eq!(json.status, 415);
        assert!(
            server.take_orders().is_empty(),
            "nothing more reached the window"
        );
        // The machine's own forms are not logged; the refused one is, as every refusal is.
        let lines = remote_lines(&ledger, 1);
        assert_eq!(lines.len(), 1, "{lines:#?}");
        assert!(
            lines[0].starts_with("no login · POST /m/p/harbor/tick · refused"),
            "{lines:#?}"
        );
    }

    fn form_headers<'a>(local: &'a str, origin: &'a str) -> [(&'a str, &'a str); 3] {
        [
            ("Host", local),
            ("Origin", origin),
            ("Content-Type", "application/x-www-form-urlencoded"),
        ]
    }

    /// The owner's form through Serve is logged before the window takes its order, and the
    /// window hears the owner's login as the caller.
    #[test]
    fn the_owners_form_through_serve_is_logged_before_its_order() {
        let scratch = crate::portfolio::tests::Scratch::new("server-pages-serve-form");
        let (mut server, port, ledger, _publisher) = serving(&scratch, Some("owner@example.com"));
        let sending = std::thread::spawn(move || {
            post_form(
                port,
                "/m/p/harbor/send",
                &[
                    ("Host", "box.example.ts.net:9443"),
                    ("X-Forwarded-Host", "box.example.ts.net:9443"),
                    ("X-Forwarded-For", "100.64.0.2"),
                    ("X-Forwarded-Proto", "https"),
                    ("Tailscale-User-Login", "owner@example.com"),
                    ("Origin", "https://box.example.ts.net:9443"),
                    ("Content-Type", "application/x-www-form-urlencoded"),
                ],
                "text=The+lock+drifted.&urgent=1",
            )
        });
        let deadline = Instant::now() + Duration::from_secs(8);
        let request = loop {
            if let Some(request) = server.take_orders().pop() {
                break request;
            }
            assert!(Instant::now() < deadline, "the order never came");
            std::thread::sleep(Duration::from_millis(10));
        };
        // Written before the window took the order: read once, at once, with no wait for it
        // (the review's: a wait would pass a line written after the order too).
        let lines: Vec<String> = Ledger::open(&ledger)
            .and_then(|ledger| ledger.events(remote::EVENT_KIND, None))
            .map(|rows| rows.into_iter().filter_map(|row| row.detail).collect())
            .unwrap_or_default();
        assert_eq!(
            lines,
            ["owner@example.com · POST /m/p/harbor/send · allowed"],
            "{lines:#?}"
        );
        assert_eq!(request.caller, "owner@example.com");
        assert_eq!(
            request.order,
            Order::Send {
                project: "harbor".to_owned(),
                text: "The lock drifted.".to_owned(),
                urgent: true
            }
        );
        request
            .reply
            .send(Ok("Sent IN-4 to harbor.".to_owned()))
            .unwrap();
        let answer = sending.join().unwrap();
        assert_eq!(answer.status, 303);
        assert!(location(&answer).starts_with("/m/p/harbor?done="));
    }

    /// The start page asks the window for its preview and draws it, a project the window does
    /// not list being a page not found; the three actions' forms reach the window as orders, a
    /// form that says neither pause nor resume never does (EP-30b).
    #[test]
    fn the_start_page_asks_the_window_and_the_actions_reach_it() {
        let scratch = crate::portfolio::tests::Scratch::new("server-pages-actions");
        let (mut server, port, _ledger, _publisher) = serving(&scratch, None);
        let local = format!("127.0.0.1:{port}");
        let origin = format!("http://{local}");
        let reading = {
            let local = local.clone();
            std::thread::spawn(move || {
                exchange(port, "GET", "/m/p/harbor/start", &[("Host", &local)])
            })
        };
        let deadline = Instant::now() + Duration::from_secs(8);
        let ask = loop {
            if let Some(ask) = server.take_asks().pop() {
                break ask;
            }
            assert!(Instant::now() < deadline, "the ask never came");
            std::thread::sleep(Duration::from_millis(10));
        };
        assert_eq!(ask.project, "harbor");
        let start = published::Start {
            project: "harbor".to_owned(),
            next: published::Next::Brief {
                id: "EP-4".to_owned(),
                size: Some("M".to_owned()),
                title: "The `scan` view".to_owned(),
            },
            model: ("claude-opus-5-5".to_owned(), "rule 1".to_owned()),
            effort: ("max".to_owned(), "rule 1".to_owned()),
            ultracode: false,
            permission_mode: ("auto".to_owned(), "the project's".to_owned()),
            downshift: None,
            run: Vec::new(),
            notes: Vec::new(),
            refusals: Vec::new(),
        };
        ask.reply.send(Some(start)).unwrap();
        let page = reading.join().unwrap();
        assert_eq!(page.status, 200, "{}", page.body);
        assert!(
            page.body.contains("Start EP-4 headless</button>"),
            "{}",
            page.body
        );
        assert!(
            page.head.contains("script-src 'self'; connect-src 'self'"),
            "{}",
            page.head
        );
        // A project the window does not list.
        let reading = {
            let local = local.clone();
            std::thread::spawn(move || {
                exchange(port, "GET", "/m/p/nowhere/start", &[("Host", &local)])
            })
        };
        let deadline = Instant::now() + Duration::from_secs(8);
        let ask = loop {
            if let Some(ask) = server.take_asks().pop() {
                break ask;
            }
            assert!(Instant::now() < deadline, "the ask never came");
            std::thread::sleep(Duration::from_millis(10));
        };
        ask.reply.send(None).unwrap();
        assert_eq!(reading.join().unwrap().status, 404);
        // The three forms, each answered as the window would.
        let headers = form_headers(&local, &origin);
        let post = |path: &'static str, body: &'static str| {
            let headers: Vec<(String, String)> = headers
                .iter()
                .map(|(n, v)| ((*n).to_owned(), (*v).to_owned()))
                .collect();
            std::thread::spawn(move || {
                let headers: Vec<(&str, &str)> = headers
                    .iter()
                    .map(|(n, v)| (n.as_str(), v.as_str()))
                    .collect();
                post_form(port, path, &headers, body)
            })
        };
        let posts = [
            post("/m/p/harbor/start", "ep=EP-4"),
            post("/m/p/harbor/stop", "key=3&ep=EP-4"),
            post("/m/autopilot", "pause=on"),
        ];
        let mut orders = obey(&mut server, 3, |order| Ok(format!("{order:?}")));
        let answers: Vec<Answer> = posts.into_iter().map(|post| post.join().unwrap()).collect();
        orders.sort_by_key(|order| format!("{order:?}"));
        assert_eq!(
            orders,
            [
                Order::Pause { paused: true },
                Order::Start {
                    project: "harbor".to_owned(),
                    ep: "EP-4".to_owned()
                },
                Order::Stop {
                    project: "harbor".to_owned(),
                    key: 3,
                    ep: "EP-4".to_owned()
                },
            ]
        );
        let backs: Vec<String> = answers.iter().map(location).collect();
        assert!(
            backs.iter().any(|back| back.starts_with("/m/?done=")),
            "{backs:?}"
        );
        assert_eq!(
            backs
                .iter()
                .filter(|back| back.starts_with("/m/p/harbor?done="))
                .count(),
            2
        );
        // The list draws the Pause's line.
        let back = backs
            .iter()
            .find(|back| back.starts_with("/m/?done="))
            .unwrap();
        let list = exchange(port, "GET", back, &[("Host", &local)]);
        assert!(
            list.body.contains(r#"<p class="said done">Pause"#),
            "{}",
            list.body
        );
        // A form that says neither, or names no run, never reaches the window.
        let none = post_form(port, "/m/autopilot", &headers, "pause=toggle");
        assert_eq!(none.status, 303);
        let none = post_form(port, "/m/p/harbor/stop", &headers, "key=x&ep=EP-4");
        assert_eq!(none.status, 303);
        assert!(server.take_orders().is_empty());
        // The script, from the exe.
        let script = exchange(port, "GET", "/m/app.js", &[("Host", &local)]);
        assert_eq!(script.status, 200);
        assert!(script.head.contains("content-type: text/javascript"));
        assert!(script.body.contains("EventSource"));
    }

    /// What a streaming answer has sent within a few seconds, read until `wanted` shows.
    fn stream_until(stream: &mut TcpStream, sent: &mut String, wanted: &str) -> bool {
        let deadline = Instant::now() + Duration::from_secs(8);
        let mut buffer = [0_u8; 4096];
        while !sent.contains(wanted) && Instant::now() < deadline {
            match stream.read(&mut buffer) {
                Ok(0) => break,
                Ok(n) => sent.push_str(&String::from_utf8_lossy(&buffer[..n])),
                Err(_) => {}
            }
        }
        sent.contains(wanted)
    }

    /// The event stream (EP-30b): an event stream's head and its first event, then a change the
    /// window publishes named by its page's path; a stranger through Serve is refused it.
    #[test]
    fn the_event_stream_names_a_change_by_its_pages_path() {
        let scratch = crate::portfolio::tests::Scratch::new("server-pages-events");
        let (_server, port, _ledger, mut publisher) = serving(&scratch, None);
        let local = format!("127.0.0.1:{port}");
        let mut stream = TcpStream::connect(("127.0.0.1", port)).unwrap();
        stream
            .set_read_timeout(Some(Duration::from_millis(200)))
            .unwrap();
        stream
            .write_all(format!("GET /m/events HTTP/1.1\r\nHost: {local}\r\n\r\n").as_bytes())
            .unwrap();
        let mut sent = String::new();
        assert!(
            stream_until(&mut stream, &mut sent, "retry: 5000"),
            "{sent}"
        );
        let head = sent.to_ascii_lowercase();
        assert!(head.starts_with("http/1.1 200"), "{sent}");
        assert!(head.contains("content-type: text/event-stream"), "{sent}");
        assert!(head.contains("cache-control: no-store"), "{sent}");
        // The window publishes a change to harbor's page.
        let current = publisher.current();
        let mut parts = crate::published::Parts {
            list: current.list.clone(),
            projects: current.projects.clone(),
            decisions: current.decisions.clone(),
        };
        parts.projects[0].resolved = 3;
        publisher.offer(parts, chrono::Utc::now());
        assert!(
            stream_until(&mut stream, &mut sent, "event: change\ndata: /m/p/harbor\n"),
            "{sent}"
        );
        // A page drawn before that change opens its stream with its version: everything, first.
        let mut late = TcpStream::connect(("127.0.0.1", port)).unwrap();
        late.set_read_timeout(Some(Duration::from_millis(200)))
            .unwrap();
        late.write_all(
            format!("GET /m/events?since=1 HTTP/1.1\r\nHost: {local}\r\n\r\n").as_bytes(),
        )
        .unwrap();
        let mut caught = String::new();
        assert!(
            stream_until(&mut late, &mut caught, "event: change\ndata: *\n"),
            "{caught}"
        );
        // A stranger through Serve gets no stream.
        let stranger = exchange(
            port,
            "GET",
            "/m/events",
            &[
                ("Host", "box.example.ts.net:9443"),
                ("Tailscale-User-Login", "someone@example.org"),
            ],
        );
        assert_eq!(stranger.status, 403);
    }

    /// While the registry names no owner, every request through Serve is refused, the owner's
    /// login too.
    #[test]
    fn no_request_passes_through_serve_while_no_owner_is_named() {
        let scratch = crate::portfolio::tests::Scratch::new("server-pages-no-owner");
        let (_server, port, ledger, _publisher) = serving(&scratch, None);
        let answer = exchange(
            port,
            "GET",
            "/m/",
            &[
                ("Host", "box.example.ts.net:9443"),
                ("Tailscale-User-Login", "owner@example.com"),
            ],
        );
        assert_eq!(answer.status, 403);
        let lines = remote_lines(&ledger, 1);
        assert_eq!(lines.len(), 1, "{lines:#?}");
        assert!(
            lines[0].ends_with("registry names no owner_login"),
            "{lines:#?}"
        );
        // The relays keep their own guard: a page's request is refused there as before.
        let local = format!("127.0.0.1:{port}");
        let health = exchange(
            port,
            "GET",
            signals::HEALTH_ROUTE,
            &[("Host", &local), ("Origin", "http://127.0.0.1")],
        );
        assert_eq!(health.status, 403);
    }
}
