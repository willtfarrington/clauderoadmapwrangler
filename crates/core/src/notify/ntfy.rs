//! The ntfy publisher (D-21; EP-20a): each toast the window's popup shows also goes to the owner's
//! phone, a message POSTed to `<host>/<topic>` on their ntfy server over the tailnet with a title
//! and a priority. The host and the topic are the registry's `ntfy_host` and `ntfy_topic`
//! settings; the publisher is off while either is missing. The request goes through WinHTTP
//! (`usage::http`), as the meter's does, so TLS, the certificate store and the proxy settings are
//! the system's. The topic, which on a server open to others works as a password, is never logged:
//! the log says `ntfy` or `ntfy failed` and why.

use std::time::Duration;

use crate::usage::http::{self, Url};

/// The longest topic ntfy takes.
pub const TOPIC_MAX: usize = 64;

/// Each step's limit (resolving, connecting, sending, receiving).
pub const TIMEOUT: Duration = Duration::from_secs(10);

/// How hard a message asks for the phone's attention: ntfy's names for its levels 5, 4 and 3.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Priority {
    Urgent,
    High,
    Default,
}

impl Priority {
    /// The `Priority` header's value.
    pub fn as_str(self) -> &'static str {
        match self {
            Priority::Urgent => "urgent",
            Priority::High => "high",
            Priority::Default => "default",
        }
    }
}

/// One message: a toast's title and text, and how hard it asks.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Push {
    pub title: String,
    pub text: String,
    pub priority: Priority,
}

impl Push {
    /// What the message says: its text, or its title when it has none (ntfy says "triggered"
    /// for an empty body).
    fn body(&self) -> &str {
        if self.text.trim().is_empty() {
            &self.title
        } else {
            &self.text
        }
    }
}

/// The owner's ntfy server and topic, from the registry. Its `Debug` names the server only.
#[derive(Clone, PartialEq, Eq)]
pub struct Ntfy {
    url: Url,
}

impl std::fmt::Debug for Ntfy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Ntfy")
            .field("server", &self.server())
            .finish_non_exhaustive()
    }
}

impl Ntfy {
    /// The publisher the settings name: `None` while the host or the topic is missing (off), and
    /// an error in words when one does not read.
    pub fn from_settings(host: Option<&str>, topic: Option<&str>) -> Option<Result<Ntfy, String>> {
        let host = host.map(str::trim).filter(|host| !host.is_empty())?;
        let topic = topic.map(str::trim).filter(|topic| !topic.is_empty())?;
        Some(Self::new(host, topic))
    }

    /// The publisher for `host` (a name or an address with an optional port, `https://` unless
    /// written `http://`) and `topic`.
    pub fn new(host: &str, topic: &str) -> Result<Ntfy, String> {
        check_topic(topic)?;
        let server = server_url(host)?;
        Ok(Ntfy {
            url: Url {
                path: format!("/{topic}"),
                ..server
            },
        })
    }

    /// The server, without the topic: `https://ntfy.example`.
    pub fn server(&self) -> String {
        let scheme = if self.url.secure { "https" } else { "http" };
        let default = if self.url.secure { 443 } else { 80 };
        if self.url.port == default {
            format!("{scheme}://{}", self.url.host)
        } else {
            format!("{scheme}://{}:{}", self.url.host, self.url.port)
        }
    }

    /// Sends `push`: the answer's status when the server took it, else why not, in words that
    /// never hold the topic. ntfy answers a publish with the message as a JSON object; any other
    /// answer, such as a web page a redirect led to, published nothing (EP-20a's review).
    pub fn publish(&self, push: &Push) -> Result<u16, String> {
        let headers = format!(
            "Title: {}\r\nPriority: {}\r\nContent-Type: text/plain; charset=utf-8\r\n",
            header_value(&push.title),
            push.priority.as_str()
        );
        let response = http::send(
            "POST",
            &self.url,
            &headers,
            push.body().as_bytes(),
            &crate::usage::agent(),
            TIMEOUT,
        )?;
        if !(200..300).contains(&response.status) {
            return Err(format!("the server answered HTTP {}", response.status));
        }
        let ntfys = serde_json::from_slice::<serde_json::Value>(&response.body)
            .is_ok_and(|answer| answer.is_object());
        if !ntfys {
            return Err(format!(
                "the server answered HTTP {} but not as ntfy does: nothing was published",
                response.status
            ));
        }
        Ok(response.status)
    }
}

/// Whether `topic` is one ntfy takes: letters, digits, `-` and `_`, up to [`TOPIC_MAX`]. The
/// refusal never quotes the topic, which can work as a password (EP-20a's review).
pub fn check_topic(topic: &str) -> Result<(), String> {
    let fits = !topic.is_empty()
        && topic.len() <= TOPIC_MAX
        && topic
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_');
    if fits {
        Ok(())
    } else {
        Err(format!(
            "ntfy_topic takes letters, digits, - and _, up to {TOPIC_MAX} of them"
        ))
    }
}

/// The server `host` names: `https://` unless it says `http://`, a port when it gives one, and no
/// path. The refusal does not quote the host.
pub fn server_url(host: &str) -> Result<Url, String> {
    let host = host.trim();
    let wrong = || {
        "ntfy_host takes a server's name or address with an optional port, such as ntfy.example \
         or http://100.64.0.5:8080"
            .to_owned()
    };
    let written = if host.starts_with("https://") || host.starts_with("http://") {
        host.to_owned()
    } else if host.contains("://") {
        return Err(wrong());
    } else {
        format!("https://{host}")
    };
    let url = Url::parse(&written).ok_or_else(wrong)?;
    let plain = url
        .host
        .bytes()
        .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'.' | b'-' | b'_' | b'[' | b']' | b':'));
    if url.path != "/" || !plain || url.port == 0 {
        return Err(wrong());
    }
    Ok(url)
}

/// A header's value on one line: as written when it is printable ASCII, else RFC 2047's encoded
/// word, which ntfy reads (its publishing page, read 2026-11-10).
fn header_value(text: &str) -> String {
    let line = text.split_whitespace().collect::<Vec<_>>().join(" ");
    if line.bytes().all(|b| (0x20..0x7f).contains(&b)) {
        line
    } else {
        format!("=?UTF-8?B?{}?=", base64(line.as_bytes()))
    }
}

/// `bytes` in base64 with padding (RFC 4648).
fn base64(bytes: &[u8]) -> String {
    const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity(bytes.len().div_ceil(3) * 4);
    for chunk in bytes.chunks(3) {
        let byte = |i: usize| u32::from(chunk.get(i).copied().unwrap_or(0));
        let n = (byte(0) << 16) | (byte(1) << 8) | byte(2);
        for i in 0..4 {
            if i <= chunk.len() {
                out.push(char::from(ALPHABET[(n >> (18 - 6 * i)) as usize & 63]));
            } else {
                out.push('=');
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_settings_name_a_server_and_a_topic_or_nothing() {
        assert_eq!(Ntfy::from_settings(None, Some("alerts")), None);
        assert_eq!(Ntfy::from_settings(Some("ntfy.example"), None), None);
        assert_eq!(Ntfy::from_settings(Some(" "), Some("alerts")), None);
        let ntfy = Ntfy::from_settings(Some(" ntfy.example "), Some("crw-alerts_1"))
            .unwrap()
            .unwrap();
        assert_eq!(
            ntfy.url,
            Url {
                secure: true,
                host: "ntfy.example".to_owned(),
                port: 443,
                path: "/crw-alerts_1".to_owned(),
            }
        );
        assert_eq!(ntfy.server(), "https://ntfy.example");
        let plain = Ntfy::new("http://100.64.0.5:8080/", "alerts").unwrap();
        assert_eq!((plain.url.secure, plain.url.port), (false, 8080));
        assert_eq!(plain.url.path, "/alerts");
        assert_eq!(plain.server(), "http://100.64.0.5:8080");
        assert_eq!(
            Ntfy::new("box.tailnet.example:8443", "a").unwrap().server(),
            "https://box.tailnet.example:8443"
        );
        assert!(Ntfy::new("box_1.example", "a").is_ok());
        for host in [
            "ntfy.example/path",
            "ftp://ntfy.example",
            "ntfy.example:port",
            "ntfy.example:0",
            "user@ntfy.example",
            "ntfy example",
            "https://",
        ] {
            let refused = Ntfy::new(host, "alerts").unwrap_err();
            assert!(refused.starts_with("ntfy_host takes"), "{host}: {refused}");
        }
        // A refusal never quotes the topic, which can work as a password (EP-20a's review); nor
        // does the publisher's `Debug`.
        let long = "t".repeat(TOPIC_MAX + 1);
        for topic in [
            "two words",
            "a/b",
            "ünï",
            "k3Jd9x82mZ.alerts",
            long.as_str(),
        ] {
            let refused = Ntfy::new("ntfy.example", topic).unwrap_err();
            assert!(refused.starts_with("ntfy_topic takes"), "{refused}");
            assert!(!refused.contains(topic), "{refused}");
        }
        assert!(Ntfy::new("ntfy.example", &"t".repeat(TOPIC_MAX)).is_ok());
        let shown = format!("{ntfy:?}");
        assert!(shown.contains("https://ntfy.example"), "{shown}");
        assert!(!shown.contains("crw-alerts_1"), "{shown}");
    }

    #[test]
    fn a_title_beyond_ascii_is_an_encoded_word() {
        assert_eq!(
            header_value("harbor EP-4\nwaits  for you"),
            "harbor EP-4 waits for you"
        );
        // ntfy's own example: the German flag.
        assert_eq!(
            header_value("\u{1F1E9}\u{1F1EA}"),
            "=?UTF-8?B?8J+HqfCfh6o=?="
        );
        for (plain, coded) in [
            ("", ""),
            ("f", "Zg=="),
            ("fo", "Zm8="),
            ("foo", "Zm9v"),
            ("foob", "Zm9vYg=="),
            ("fooba", "Zm9vYmE="),
            ("foobar", "Zm9vYmFy"),
        ] {
            assert_eq!(base64(plain.as_bytes()), coded, "{plain}");
        }
        assert_eq!(Priority::Urgent.as_str(), "urgent");
        assert_eq!(Priority::High.as_str(), "high");
        assert_eq!(Priority::Default.as_str(), "default");
    }

    /// ntfy's answer to a publish: the message as JSON.
    #[cfg(windows)]
    const ANSWER: &str = r#"{"id":"hwQ2YpKdmg6p","time":1762816800,"event":"message","topic":"crw-alerts","message":"Q?"}"#;

    #[cfg(windows)]
    #[test]
    fn a_push_is_a_post_with_its_title_and_priority() {
        use crate::signals::relay::tests::{answer_once, answer_once_with};
        use std::net::TcpListener;

        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        let ntfy = Ntfy::new(&format!("http://127.0.0.1:{port}"), "crw-alerts").unwrap();
        let server = std::thread::spawn(move || answer_once_with(&listener, 200, ANSWER));
        let push = Push {
            title: "harbor EP-4 waits for you".to_owned(),
            text: "Which colour · the berth's?".to_owned(),
            priority: Priority::High,
        };
        assert_eq!(ntfy.publish(&push), Ok(200));
        let sent = server.join().unwrap();
        let lower = sent.to_lowercase();
        assert!(lower.starts_with("post /crw-alerts http/1.1\r\n"), "{sent}");
        assert!(
            lower.contains("\r\ntitle: harbor ep-4 waits for you\r\n"),
            "{sent}"
        );
        assert!(lower.contains("\r\npriority: high\r\n"), "{sent}");
        assert!(
            sent.ends_with("\r\n\r\nWhich colour · the berth's?"),
            "{sent}"
        );
        // A message with no text says its title; a refusal is said with its status.
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        let ntfy = Ntfy::new(&format!("http://127.0.0.1:{port}"), "crw-alerts").unwrap();
        let server = std::thread::spawn(move || answer_once(&listener, 403));
        let bare = Push {
            title: "harbor: the chain ends after EP-5".to_owned(),
            text: String::new(),
            priority: Priority::Default,
        };
        assert_eq!(
            ntfy.publish(&bare),
            Err("the server answered HTTP 403".to_owned())
        );
        let sent = server.join().unwrap();
        assert!(
            sent.ends_with("\r\n\r\nharbor: the chain ends after EP-5"),
            "{sent}"
        );
        assert!(
            sent.to_lowercase().contains("\r\npriority: default\r\n"),
            "{sent}"
        );
        // A 2xx that is not ntfy's answer (a web page a redirect led to) published nothing.
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        let ntfy = Ntfy::new(&format!("http://127.0.0.1:{port}"), "crw-alerts").unwrap();
        let server = std::thread::spawn(move || {
            answer_once_with(&listener, 200, "<!DOCTYPE html><title>ntfy</title>")
        });
        let refused = ntfy.publish(&bare).unwrap_err();
        assert!(refused.contains("not as ntfy does"), "{refused}");
        server.join().unwrap();
    }
}
