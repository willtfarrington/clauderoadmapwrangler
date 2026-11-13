//! The loopback server (D-14, D-20, D-28; EP-12): axum on 127.0.0.1 at the registry's port, on a
//! thread with a small runtime of its own, so the frame loop sees a channel. `POST /hook` and
//! `POST /statusline` take what `crw hook` and `crw statusline` relay and hand it to the window
//! with a repaint; `GET /health` names the app (single instance probes it at EP-28). Loopback
//! callers are trusted (D-28), and only they get through: a request that carries an `Origin` (a web
//! page's) or names another host than the loopback (a page whose name was pointed at 127.0.0.1) is
//! refused, and a body is JSON, which a page cannot send across origins without asking first.
//! `POST /restart` (EP-37) is `crw install`'s request that the window restart into the installed
//! build: the window answers it at its next pass, and the server waits for that answer a few
//! seconds. The phone's routes join at EP-30.

use std::net::Ipv4Addr;
use std::sync::Arc;
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;
use std::time::Duration;

use axum::extract::{DefaultBodyLimit, State};
use axum::http::{HeaderMap, StatusCode, header};
use axum::routing::{get, post};
use axum::{Json, Router};
use crw_core::signals::{self, Hook, RestartAnswer, RestartAsk, Signal, StatusLine};
use serde::Serialize;
use tokio::sync::oneshot;

/// The largest body taken: the relays' forms are a few hundred bytes.
const BODY_LIMIT: usize = 256 << 10;

/// How long a restart request waits for the window's answer, within `crw install`'s budget.
const RESTART_WAIT: Duration = Duration::from_secs(6);

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
}

impl Server {
    /// Starts listening on `127.0.0.1:<port>` (0 for any free port, in the tests); `wake` asks the
    /// window for a frame.
    pub fn start(port: u16, wake: impl Fn() + Send + Sync + 'static) -> Server {
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
    Router::new()
        .route(signals::HOOK_ROUTE, post(hook))
        .route(signals::STATUS_ROUTE, post(status_line))
        .route(signals::HEALTH_ROUTE, get(health))
        .route(signals::RESTART_ROUTE, post(restart))
        .route(signals::SHOW_ROUTE, post(show))
        .layer(DefaultBodyLimit::max(BODY_LIMIT))
        .with_state(shared)
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
        let Some(host) = headers
            .get(header::HOST)
            .and_then(|host| host.to_str().ok())
        else {
            return false;
        };
        let host = host.trim().to_ascii_lowercase();
        host == format!("127.0.0.1:{}", self.port) || host == format!("localhost:{}", self.port)
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
        assert_eq!(signals::ask_show(port), signals::Yield::Shown);
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
        assert_eq!(signals::ask_show(free), signals::Yield::NoWindow);
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
}
