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

use std::net::Ipv4Addr;
use std::path::PathBuf;
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use axum::extract::{DefaultBodyLimit, Request, State};
use axum::http::{HeaderMap, HeaderValue, StatusCode, header};
use axum::middleware::{self, Next};
use axum::response::{Html, IntoResponse, Redirect, Response};
use axum::routing::{any, get, post};
use axum::{Json, Router};
use crw_core::ledger::Ledger;
use crw_core::registry::Store;
use crw_core::remote::{self, Asked, Throttle};
use crw_core::signals::{self, Hook, RestartAnswer, RestartAsk, Signal, StatusLine};
use serde::Serialize;
use tokio::sync::oneshot;

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

/// What a page may load: its own stylesheet and its own forms, nothing else (EP-30b adds its own
/// script).
const CONTENT_POLICY: &str = "default-src 'none'; style-src 'self'; form-action 'self'; \
                              frame-ancestors 'none'; base-uri 'none'";

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
    bound: Receiver<Result<u16, Listening>>,
    pub listening: Listening,
}

struct Shared {
    port: u16,
    sender: Sender<Signal>,
    restarts: Sender<RestartRequest>,
    wake: Wake,
    /// The phone's pages' needs; none in the relays' tests, whose `/m/` pages say they are not
    /// served.
    pages: Option<Pages>,
    /// The refused identities' last lines (EP-30).
    refused: Mutex<Throttle>,
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
                        wake,
                        pages,
                        refused: Mutex::new(Throttle::default()),
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
}

fn router(shared: Arc<Shared>) -> Router {
    // The phone's pages, each behind the identity check (EP-30).
    let pages = Router::new()
        .route(PAGES_BARE, get(to_list))
        .route(PAGES_ROOT, get(list_page))
        .route(STYLESHEET_ROUTE, get(stylesheet))
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
/// held is old.
async fn list_page(State(shared): State<Arc<Shared>>) -> Response {
    let Some(pages) = &shared.pages else {
        return page(
            StatusCode::SERVICE_UNAVAILABLE,
            "Not served",
            "This server does not serve the pages.",
        );
    };
    let published = pages.published.fresh(|| (shared.wake)()).await;
    match crate::pages::list(&published) {
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

/// Any other `/m/…` path, once it passed the check.
async fn not_found() -> Response {
    page(
        StatusCode::NOT_FOUND,
        "Not found",
        "No page here: the project list is at /m/.",
    )
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
        publisher.offer(
            List {
                groups: vec![Group {
                    state: State::Active,
                    projects: vec![harbor],
                }],
                ..List::default()
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
