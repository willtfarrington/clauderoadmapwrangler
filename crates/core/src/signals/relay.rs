//! The relays' side of the loopback (D-28): one HTTP/1.1 request to 127.0.0.1 over a plain socket,
//! `Connection: close`, within a budget; no proxy, no TLS and no client crate, so a relay starts
//! and finishes in a few milliseconds. `crw signals status` asks `/health` the same way.

use std::io::{Read, Write};
use std::net::{Ipv4Addr, SocketAddr, TcpStream};
use std::time::{Duration, Instant};

/// The most of an answer read.
const ANSWER_CAP: usize = 64 << 10;

/// The connection's share of the budget. A window that listens has its connections completed by
/// the system at once, whatever the window is doing; a port nobody listens on is refused on
/// Windows only after the system's own retries (the relays took the whole half second on
/// 2026-11-02), so with no window running a relay costs this much and no more.
pub const CONNECT: Duration = Duration::from_millis(50);

/// The window's answer: its status code and its body.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Answer {
    pub status: u16,
    pub body: Vec<u8>,
}

/// Sends `method route` to the window on `port`, with `body` as JSON when there is one, and reads
/// its answer, all within `budget`.
pub fn request(
    port: u16,
    method: &str,
    route: &str,
    body: Option<&[u8]>,
    budget: Duration,
) -> Result<Answer, String> {
    let deadline = Instant::now() + budget;
    let left = || {
        deadline
            .checked_duration_since(Instant::now())
            .filter(|left| !left.is_zero())
            .ok_or_else(|| format!("no answer within {budget:?}"))
    };
    let address = SocketAddr::from((Ipv4Addr::LOCALHOST, port));
    let fail = |e: std::io::Error| format!("127.0.0.1:{port}: {e}");
    let mut stream = TcpStream::connect_timeout(&address, left()?.min(CONNECT)).map_err(fail)?;
    stream.set_nodelay(true).map_err(fail)?;
    let mut head =
        format!("{method} {route} HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\nConnection: close\r\n");
    if let Some(body) = body {
        head.push_str(&format!(
            "Content-Type: application/json\r\nContent-Length: {}\r\n",
            body.len()
        ));
    }
    head.push_str("\r\n");
    stream.set_write_timeout(Some(left()?)).map_err(fail)?;
    stream.write_all(head.as_bytes()).map_err(fail)?;
    if let Some(body) = body {
        stream.write_all(body).map_err(fail)?;
    }
    let mut answer = Vec::new();
    let mut buffer = [0_u8; 4096];
    loop {
        stream.set_read_timeout(Some(left()?)).map_err(fail)?;
        match stream.read(&mut buffer) {
            Ok(0) => break,
            Ok(n) => {
                answer.extend_from_slice(&buffer[..n]);
                if answer.len() > ANSWER_CAP {
                    break;
                }
            }
            Err(e) => return Err(fail(e)),
        }
    }
    parse(&answer).ok_or_else(|| format!("127.0.0.1:{port}: an answer that is not HTTP"))
}

/// The status code and the body of an answer read to its end.
fn parse(answer: &[u8]) -> Option<Answer> {
    let split = answer.windows(4).position(|w| w == b"\r\n\r\n")?;
    let head = std::str::from_utf8(&answer[..split]).ok()?;
    let status = head
        .lines()
        .next()?
        .strip_prefix("HTTP/1.")?
        .get(2..)?
        .split_whitespace()
        .next()?
        .parse()
        .ok()?;
    Some(Answer {
        status,
        body: answer[split + 4..].to_vec(),
    })
}

#[cfg(test)]
pub(crate) mod tests {
    use std::net::TcpListener;

    use super::*;

    /// Accepts one request on `listener`, answers it with `status` and an empty body, and returns
    /// what it was sent.
    pub(crate) fn answer_once(listener: &TcpListener, status: u16) -> String {
        answer_once_with(listener, status, "")
    }

    /// The same, answering with `body` (ntfy's tests, EP-20a).
    pub(crate) fn answer_once_with(listener: &TcpListener, status: u16, body: &str) -> String {
        let (mut stream, _) = listener.accept().unwrap();
        stream
            .set_read_timeout(Some(Duration::from_secs(5)))
            .unwrap();
        let mut request = Vec::new();
        let mut buffer = [0_u8; 4096];
        // The head, then as many bytes as it announces.
        loop {
            let n = stream.read(&mut buffer).unwrap();
            request.extend_from_slice(&buffer[..n]);
            if let Some(split) = request.windows(4).position(|w| w == b"\r\n\r\n") {
                let head = String::from_utf8_lossy(&request[..split]).to_lowercase();
                let length: usize = head
                    .lines()
                    .find_map(|line| line.strip_prefix("content-length:"))
                    .map_or(0, |n| n.trim().parse().unwrap());
                if request.len() >= split + 4 + length {
                    break;
                }
            }
            if n == 0 {
                break;
            }
        }
        write!(
            stream,
            "HTTP/1.1 {status} Whatever\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
            body.len()
        )
        .unwrap();
        String::from_utf8(request).unwrap()
    }

    #[test]
    fn reads_the_status_and_the_body() {
        assert_eq!(
            parse(b"HTTP/1.1 200 OK\r\ncontent-length: 2\r\n\r\n{}"),
            Some(Answer {
                status: 200,
                body: b"{}".to_vec()
            })
        );
        assert_eq!(
            parse(b"HTTP/1.1 204 No Content\r\n\r\n").unwrap().status,
            204
        );
        assert_eq!(parse(b"SSH-2.0-OpenSSH\r\n\r\n"), None);
        assert_eq!(parse(b"HTTP/1.1 200 OK\r\n"), None);
    }

    #[test]
    fn a_request_without_a_body_names_no_type() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        let server = std::thread::spawn(move || answer_once(&listener, 200));
        let answer = request(port, "GET", "/health", None, Duration::from_secs(2)).unwrap();
        assert_eq!(answer.status, 200);
        let sent = server.join().unwrap();
        assert_eq!(
            sent,
            format!("GET /health HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\nConnection: close\r\n\r\n")
        );
    }

    #[test]
    fn a_window_that_never_answers_costs_the_budget_only() {
        // A listener that accepts and says nothing.
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        let started = Instant::now();
        let result = request(
            port,
            "POST",
            "/hook",
            Some(b"{}"),
            Duration::from_millis(300),
        );
        assert!(result.is_err(), "{result:?}");
        let took = started.elapsed();
        assert!(took >= Duration::from_millis(250), "{took:?}");
        assert!(took < Duration::from_millis(900), "{took:?}");
        drop(listener);
    }
}
