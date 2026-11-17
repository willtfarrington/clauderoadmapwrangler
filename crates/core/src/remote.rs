//! The phone's pages' guard (D-28; EP-30): who a request to the window's `/m/` routes comes from,
//! told by its headers alone, and the words of the remote-action log.
//!
//! The window's server binds the loopback only, so a request reaches it from the machine itself or
//! through Tailscale Serve, which proxies the phone's path to it (EP-31a). Serve removes any
//! identity header a request carries and sets `Tailscale-User-Login` to the login of the user whose
//! device made it; a device with no user (a tagged one) gets none. It also adds the proxy's own
//! headers (`X-Forwarded-Host`, `X-Forwarded-For`, `X-Forwarded-Proto`). So:
//!
//! - a request that carries the login came through Serve, and passes only when the login is the
//!   registry's `owner_login` (ASCII case aside) and its `Host` names a tailnet machine or the
//!   loopback. A page that reaches the loopback through a name of its own pointed at 127.0.0.1
//!   (DNS rebinding) is same-origin to the browser and may set the login itself, but its `Host`
//!   is its own name. While the registry names no owner, or does not read, every such request is
//!   refused;
//! - a request without the login is local only when its `Host` names the loopback at the window's
//!   port and it carries no header a proxy adds, so a request through Serve from a device with no
//!   user is refused whichever way Serve passes `Host`;
//! - an `Origin` other than the page's own is refused, and a form (any method but GET and HEAD)
//!   must name its own, which a browser sends with every POST.
//!
//! Every remote action is a line of the ledger's `event` table of kind [`EVENT_KIND`], written
//! before it runs ([`allowed_line`]); a refused request is a line too ([`refused_line`]), at most
//! once a minute for each identity ([`Throttle`]). A line names the identity, the method, the path
//! and what became of the request, and of the headers only whether a proxy's were present, never
//! a value beyond the login (D-3).
//!
//! EP-31a: while the registry's `remote_view` names this machine (Enable remote view wrote it), a
//! request with the login passes only with that name or the loopback in its `Host`, no other
//! tailnet machine's; and the first request through Serve a window allows is a line of its own
//! ([`arrival_line`]) saying how Serve passed `Host` and which proxy headers came, so that the
//! owner's first real request shows which of the two readings the guard keeps is Serve's.

use std::collections::HashMap;
use std::fmt;
use std::time::{Duration, Instant};

/// The ledger's `event` kind of the remote-action log.
pub const EVENT_KIND: &str = "remote";

/// The header Serve sets to the login of the user whose device made the request.
pub const LOGIN_HEADER: &str = "tailscale-user-login";

/// How long a refused identity's line keeps another from the log.
pub const REFUSAL_EVERY: Duration = Duration::from_secs(60);

/// The most refused identities a minute logs: a flood of made-up logins, which only a process on
/// the machine itself could send, does not fill the ledger.
const REFUSED_IDENTITIES: usize = 64;

/// The longest login the setting takes.
const LOGIN_CHARS: usize = 254;

/// What the guard reads of a request: its method, its path and the headers that tell where it
/// came from.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Asked {
    pub method: String,
    pub path: String,
    pub host: Option<String>,
    pub origin: Option<String>,
    pub login: Option<String>,
    /// The first host `X-Forwarded-Host` names: the page's own, through Serve.
    pub forwarded_host: Option<String>,
    /// The first scheme `X-Forwarded-Proto` names.
    pub forwarded_proto: Option<String>,
    /// The headers a proxy adds that the request carried, by name in lower case: their presence
    /// alone is kept.
    pub proxied: Vec<String>,
}

impl Asked {
    /// The request's method, its path (no query) and its headers as name and value; a header's
    /// first value counts, and a value that is not text reads as empty.
    pub fn new<'a>(
        method: &str,
        path: &str,
        headers: impl IntoIterator<Item = (&'a str, &'a str)>,
    ) -> Self {
        // As sent: a method's name is case-sensitive, so `get` is no GET (the review's).
        let mut asked = Self {
            method: method.to_owned(),
            path: path.to_owned(),
            ..Self::default()
        };
        for (name, value) in headers {
            let name = name.trim().to_ascii_lowercase();
            let value = value.trim();
            let first = |slot: &mut Option<String>, value: &str| {
                if slot.is_none() {
                    *slot = Some(value.to_owned());
                }
            };
            match name.as_str() {
                "host" => first(&mut asked.host, value),
                "origin" => first(&mut asked.origin, value),
                LOGIN_HEADER => first(&mut asked.login, value),
                _ => {}
            }
            if name == "x-forwarded-host" {
                first(&mut asked.forwarded_host, list_head(value));
            }
            if name == "x-forwarded-proto" {
                first(&mut asked.forwarded_proto, list_head(value));
            }
            if is_proxy_header(&name) && !asked.proxied.contains(&name) {
                asked.proxied.push(name);
            }
        }
        asked
    }

    /// GET and HEAD change nothing; any other method is a form, which must name its origin.
    pub fn is_safe(&self) -> bool {
        matches!(self.method.as_str(), "GET" | "HEAD")
    }
}

/// The first item of a header's comma-separated list.
fn list_head(value: &str) -> &str {
    value.split(',').next().unwrap_or_default().trim()
}

/// A header a proxy adds, which a request from the machine itself never carries: the forwarding
/// headers, and Serve's own beside the login.
fn is_proxy_header(name: &str) -> bool {
    name.starts_with("x-forwarded-")
        || name == "forwarded"
        || name == "via"
        || name == "x-real-ip"
        || (name.starts_with("tailscale-") && name != LOGIN_HEADER)
}

/// Whether `host`, a `Host` header, names the loopback at `port` (`127.0.0.1:<port>` or
/// `localhost:<port>`): the relays' guard and the pages' local rule (EP-12, EP-30).
pub fn is_loopback(host: &str, port: u16) -> bool {
    let host = host.trim().to_ascii_lowercase();
    host == format!("127.0.0.1:{port}") || host == format!("localhost:{port}")
}

/// Whether `host`, a `Host` header, names a machine of a tailnet (`<machine>.<tailnet>.ts.net`,
/// with or without a port): the name a phone asks Serve for, which Serve hands on. Nobody can
/// point such a name at 127.0.0.1, so a page that reaches the loopback through a name of its own
/// (DNS rebinding) never carries one.
pub fn is_tailnet_name(host: &str) -> bool {
    let host = host.trim().to_ascii_lowercase();
    let name = match host.rsplit_once(':') {
        Some((name, port)) if !port.is_empty() && port.bytes().all(|b| b.is_ascii_digit()) => name,
        Some(_) => return false,
        None => host.as_str(),
    };
    let Some(machine) = name.strip_suffix(".ts.net") else {
        return false;
    };
    let labels: Vec<&str> = machine.split('.').collect();
    labels.len() >= 2
        && labels.iter().all(|label| {
            !label.is_empty()
                && label
                    .bytes()
                    .all(|b| b.is_ascii_alphanumeric() || b == b'-')
        })
}

/// Whether `host`, a `Host` header, names `machine` (a tailnet name without a port), with or
/// without a port, ASCII case aside; a final dot on either side is no difference (EP-31a).
pub fn names_machine(host: &str, machine: &str) -> bool {
    let host = host.trim();
    let name = match host.rsplit_once(':') {
        Some((name, port)) if port.bytes().all(|b| b.is_ascii_digit()) => name,
        _ => host,
    };
    let bare = |name: &str| name.trim().trim_end_matches('.').to_ascii_lowercase();
    !machine.trim().is_empty() && bare(name) == bare(machine)
}

/// What the guard reads of the registry for a request through Serve: the owner's login, and
/// this machine's tailnet name while remote view is on (`remote_view`, EP-31a).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Owner {
    pub login: Option<String>,
    pub machine: Option<String>,
}

/// Who a request comes from, once it passed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Caller {
    /// The machine itself: a loopback `Host`, no login and no proxy's header.
    Local,
    /// The owner, through Serve: the login the registry names, as the request carried it.
    Owner(String),
}

impl Caller {
    /// Whether the request came through Serve, so that an action of it is logged.
    pub fn is_remote(&self) -> bool {
        matches!(self, Caller::Owner(_))
    }

    /// The identity in the log's and the window's words.
    pub fn identity(&self) -> &str {
        match self {
            Caller::Local => "this machine",
            Caller::Owner(login) => login,
        }
    }
}

/// Why a request was refused, in the log's words; the refused caller's page names no reason.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Refusal {
    /// Through Serve while the registry names no `owner_login`.
    NoOwner,
    /// Through Serve while the registry does not read.
    RegistryUnread,
    /// Through Serve with another login than the owner's.
    NotTheOwner,
    /// A login with a `Host` that is neither the loopback at the window's port nor a tailnet
    /// name: a page that reached the loopback through a name of its own (DNS rebinding) and set
    /// the header itself.
    ForeignHost,
    /// A login with another tailnet machine's name in `Host` than the one the registry's
    /// `remote_view` names (EP-31a).
    OtherMachine,
    /// No login, and not from the machine itself: the `Host` is not the loopback at the window's
    /// port, or a proxy's headers came with it (by name).
    NotLocal {
        loopback_host: bool,
        proxied: Vec<String>,
    },
    /// An `Origin` other than the page's own.
    ForeignOrigin,
    /// A form that names no origin.
    NoOrigin,
}

impl fmt::Display for Refusal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Refusal::NoOwner => f.write_str("through Serve, and the registry names no owner_login"),
            Refusal::RegistryUnread => f.write_str("through Serve, and the registry did not read"),
            Refusal::NotTheOwner => {
                f.write_str("through Serve with another login than the owner's")
            }
            Refusal::ForeignHost => {
                f.write_str("a login with a Host neither the loopback nor a tailnet name")
            }
            Refusal::OtherMachine => {
                f.write_str("a login with another tailnet machine's name in Host than this one's")
            }
            Refusal::NotLocal {
                loopback_host,
                proxied,
            } => {
                f.write_str("no login, and not from this machine: ")?;
                let host = if *loopback_host {
                    "Host the loopback"
                } else {
                    "Host not the loopback"
                };
                if proxied.is_empty() {
                    write!(f, "{host}, no proxy header")
                } else {
                    write!(f, "{host}, proxy headers {}", proxied.join(", "))
                }
            }
            Refusal::ForeignOrigin => f.write_str("an Origin other than the page's own"),
            Refusal::NoOrigin => f.write_str("a form that names no Origin"),
        }
    }
}

/// The guard: who `asked` comes from, `port` being the window's. `owner` reads the registry's
/// `owner_login` and `remote_view` (an error when the registry does not read), and is asked only
/// for a request through Serve.
pub fn check(
    asked: &Asked,
    port: u16,
    owner: impl FnOnce() -> Result<Owner, String>,
) -> Result<Caller, Refusal> {
    let caller = match asked.login.as_deref() {
        Some(login) => {
            // A page served from a name of its own that points at the loopback is same-origin to
            // the browser, so it may set the login and the forwarding headers itself: only Serve's
            // `Host` (the tailnet name, or the loopback's if Serve rewrote it) passes.
            let host = asked.host.as_deref().unwrap_or_default();
            if !is_loopback(host, port) && !is_tailnet_name(host) {
                return Err(Refusal::ForeignHost);
            }
            let owner = owner().map_err(|_| Refusal::RegistryUnread)?;
            // While remote view is on, this machine's own name alone (EP-31a): Serve hands on the
            // name the phone asked for, which is this machine's, in `Host`, or in
            // `X-Forwarded-Host` when it passes the loopback's `Host` (the review's: the
            // narrowing must hold whichever way Serve passes it).
            if let Some(machine) = owner.machine.as_deref() {
                let named = if is_tailnet_name(host) {
                    Some(host)
                } else {
                    asked.forwarded_host.as_deref()
                };
                if named.is_some_and(|name| !names_machine(name, machine)) {
                    return Err(Refusal::OtherMachine);
                }
            }
            let owner = owner.login.ok_or(Refusal::NoOwner)?;
            let login = login.trim();
            if login.is_empty() || !login.eq_ignore_ascii_case(owner.trim()) {
                return Err(Refusal::NotTheOwner);
            }
            Caller::Owner(login.to_owned())
        }
        None => {
            let loopback_host = asked
                .host
                .as_deref()
                .is_some_and(|host| is_loopback(host, port));
            if !loopback_host || !asked.proxied.is_empty() {
                return Err(Refusal::NotLocal {
                    loopback_host,
                    proxied: asked.proxied.clone(),
                });
            }
            Caller::Local
        }
    };
    match asked.origin.as_deref() {
        Some(origin) => {
            let own = own_origin(asked, &caller);
            if own.is_some_and(|own| origin.trim().eq_ignore_ascii_case(&own)) {
                Ok(caller)
            } else {
                Err(Refusal::ForeignOrigin)
            }
        }
        None if asked.is_safe() => Ok(caller),
        None => Err(Refusal::NoOrigin),
    }
}

/// The page's own origin as a browser names it: `http://` and the loopback `Host` for the machine
/// itself. Through Serve, the scheme `X-Forwarded-Proto` names (`https` without it) and the host
/// the `Host` names when it is the tailnet name, or `X-Forwarded-Host` when Serve passed the
/// loopback's `Host` (a page cannot send that `Host` with headers of its own).
fn own_origin(asked: &Asked, caller: &Caller) -> Option<String> {
    let host = asked.host.as_deref()?.trim();
    match caller {
        Caller::Local => Some(format!("http://{host}")),
        Caller::Owner(_) => {
            let host = if is_tailnet_name(host) {
                host
            } else {
                asked.forwarded_host.as_deref()?.trim()
            };
            let scheme = asked.forwarded_proto.as_deref().unwrap_or("https").trim();
            (!host.is_empty()).then(|| format!("{scheme}://{host}"))
        }
    }
}

/// The login the setting takes (EP-30): one word, no space and no control character, at most
/// 254 characters; trimmed.
pub fn check_login(value: &str) -> Result<&str, String> {
    let login = value.trim();
    if login.is_empty() {
        return Err("owner_login needs a login, such as you@example.com".to_owned());
    }
    if login.chars().any(|c| c.is_whitespace() || c.is_control()) {
        return Err(format!(
            "owner_login is one word with no space or control character, not {value:?}"
        ));
    }
    if login.chars().count() > LOGIN_CHARS {
        return Err(format!(
            "owner_login takes at most {LOGIN_CHARS} characters"
        ));
    }
    Ok(login)
}

/// The name the registry's `remote_view` takes (EP-31a): a tailnet machine's name with no port,
/// trimmed, without the final dot `tailscale status` writes.
pub fn check_machine(value: &str) -> Result<String, String> {
    let name = value.trim().trim_end_matches('.');
    if name.contains(':') || !is_tailnet_name(name) {
        return Err(format!(
            "remote_view takes this machine's tailnet name with no port, such as \
             box.example.ts.net, not {value:?}"
        ));
    }
    Ok(name.to_owned())
}

/// The log's line: the identity, the method and the path, and what became of the request.
fn line(identity: &str, asked: &Asked, verdict: &str) -> String {
    format!("{identity} · {} {} · {verdict}", asked.method, asked.path)
}

/// The line of an action through Serve, written before it runs.
pub fn allowed_line(caller: &Caller, asked: &Asked) -> String {
    line(caller.identity(), asked, "allowed")
}

/// The line of the first request through Serve a window allows (EP-31a): how Serve passed `Host`
/// (this machine's name as the registry's `remote_view` names it, another tailnet name, the
/// loopback), with or without a port, and which proxy headers came, by name. No value beyond the
/// login: the owner task's first step reads which reading the real request showed.
pub fn arrival_line(caller: &Caller, asked: &Asked, port: u16, machine: Option<&str>) -> String {
    let host = asked.host.as_deref().unwrap_or_default().trim();
    let form = if is_loopback(host, port) {
        "the loopback"
    } else if machine.is_some_and(|machine| names_machine(host, machine)) {
        "this machine's tailnet name"
    } else if is_tailnet_name(host) {
        "a tailnet name"
    } else {
        "another name"
    };
    let with_port = host
        .rsplit_once(':')
        .is_some_and(|(_, port)| !port.is_empty() && port.bytes().all(|b| b.is_ascii_digit()));
    let port_words = if with_port {
        "with a port"
    } else {
        "with no port"
    };
    let headers = if asked.proxied.is_empty() {
        "no proxy header".to_owned()
    } else {
        format!("proxy headers {}", asked.proxied.join(", "))
    };
    line(
        caller.identity(),
        asked,
        &format!(
            "allowed, the first request through Serve since the window started: Host \
             {form} {port_words}, {headers}"
        ),
    )
}

/// The identity a refused request is logged and throttled under: its login, or `no login`.
pub fn refused_identity(asked: &Asked) -> String {
    match asked.login.as_deref().map(str::trim) {
        Some(login) if !login.is_empty() => login.to_owned(),
        _ => "no login".to_owned(),
    }
}

/// The line of a refused request.
pub fn refused_line(asked: &Asked, refusal: &Refusal) -> String {
    line(
        &refused_identity(asked),
        asked,
        &format!("refused: {refusal}"),
    )
}

/// The refused identities' last lines, so that each is logged at most once a minute.
#[derive(Debug, Default)]
pub struct Throttle {
    last: HashMap<String, Instant>,
}

impl Throttle {
    /// Whether a refusal of `identity` at `now` is due a line; `true` marks it so.
    pub fn due(&mut self, identity: &str, now: Instant) -> bool {
        self.last
            .retain(|_, at| now.saturating_duration_since(*at) < REFUSAL_EVERY);
        let key = identity.to_ascii_lowercase();
        if self.last.contains_key(&key) || self.last.len() >= REFUSED_IDENTITIES {
            return false;
        }
        self.last.insert(key, now);
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const PORT: u16 = 47831;
    const OWNER: &str = "owner@example.com";

    fn asked(method: &str, headers: &[(&str, &str)]) -> Asked {
        Asked::new(method, "/m/", headers.iter().copied())
    }

    fn owner() -> Result<Owner, String> {
        Ok(Owner {
            login: Some(OWNER.to_owned()),
            machine: None,
        })
    }

    fn never() -> Result<Owner, String> {
        panic!("the registry is read only for a request through Serve")
    }

    #[test]
    fn reads_the_headers_that_tell_where_a_request_came_from() {
        let read = asked(
            "GET",
            &[
                ("Host", " 127.0.0.1:47831 "),
                ("X-Forwarded-Host", "box.example.ts.net:9443, other"),
                ("X-Forwarded-For", "100.64.0.2"),
                ("X-Forwarded-Proto", "https"),
                ("Tailscale-User-Login", OWNER),
                ("Tailscale-User-Name", "Owner"),
                ("tailscale-user-name", "again"),
                ("Accept", "text/html"),
            ],
        );
        assert_eq!(read.method, "GET");
        assert_eq!(read.host.as_deref(), Some("127.0.0.1:47831"));
        assert_eq!(
            read.forwarded_host.as_deref(),
            Some("box.example.ts.net:9443")
        );
        assert_eq!(read.forwarded_proto.as_deref(), Some("https"));
        assert_eq!(read.login.as_deref(), Some(OWNER));
        assert_eq!(
            read.proxied,
            [
                "x-forwarded-host",
                "x-forwarded-for",
                "x-forwarded-proto",
                "tailscale-user-name"
            ],
            "by name, once each; the login is no proxy header"
        );
        assert!(read.is_safe());
        assert!(!asked("POST", &[]).is_safe());
        assert!(asked("HEAD", &[]).is_safe());
        // A method's name is case-sensitive: an extension method `get` is a form.
        assert!(!asked("get", &[]).is_safe());
    }

    #[test]
    fn a_tailnet_name_is_a_machine_of_a_tailnet_with_or_without_its_port() {
        for name in [
            "box.example.ts.net:9443",
            "box.example.ts.net",
            "BOX.Tail-1a2b.TS.NET:443",
        ] {
            assert!(is_tailnet_name(name), "{name}");
        }
        for name in [
            "ts.net",
            "example.ts.net",
            ".example.ts.net",
            "box..ts.net",
            "box.example.ts.net:",
            "box.example.ts.net:94x3",
            "box.example.ts.net.attacker.example",
            "box.example.ts.net.attacker.example:47831",
            "attacker.example:47831",
            "127.0.0.1:47831",
            "box_1.example.ts.net",
        ] {
            assert!(!is_tailnet_name(name), "{name}");
        }
    }

    /// A page served from a name of its own that points at the loopback (DNS rebinding) is
    /// same-origin to the browser: it may send the owner's login and forwarding headers of its
    /// choosing, and no `Origin` on a GET. Its `Host` gives it away (the review's finding).
    #[test]
    fn a_page_that_reaches_the_loopback_through_its_own_name_is_refused_with_the_login() {
        let rebinding = asked(
            "GET",
            &[
                ("Host", "attacker.example:47831"),
                ("Tailscale-User-Login", OWNER),
            ],
        );
        assert_eq!(check(&rebinding, PORT, never), Err(Refusal::ForeignHost));
        // A form that names its own origin and forges the forwarding headers to match.
        let form = asked(
            "POST",
            &[
                ("Host", "attacker.example:47831"),
                ("Origin", "http://attacker.example:47831"),
                ("Tailscale-User-Login", OWNER),
                ("X-Forwarded-Proto", "http"),
                ("X-Forwarded-Host", "attacker.example:47831"),
            ],
        );
        assert_eq!(check(&form, PORT, never), Err(Refusal::ForeignHost));
        // A tailnet name as Serve hands it on passes; with Serve's loopback `Host` the
        // forwarded host names the page's origin, which a page cannot send with that `Host`.
        let forwarded = asked(
            "POST",
            &[
                ("Host", "box.example.ts.net:9443"),
                ("X-Forwarded-Host", "attacker.example:47831"),
                ("Origin", "https://attacker.example:47831"),
                ("Tailscale-User-Login", OWNER),
            ],
        );
        assert_eq!(
            check(&forwarded, PORT, owner),
            Err(Refusal::ForeignOrigin),
            "with a tailnet Host, the origin's host is the Host's"
        );
    }

    #[test]
    fn the_machine_itself_is_local_by_the_loopback_host_alone() {
        for host in ["127.0.0.1:47831", "LOCALHOST:47831"] {
            let local = asked("GET", &[("Host", host)]);
            assert_eq!(check(&local, PORT, never), Ok(Caller::Local), "{host}");
        }
        // Another port, a name pointed at the loopback, no Host at all.
        for host in ["127.0.0.1:47832", "evil.example:47831", "[::1]:47831"] {
            let refused = check(&asked("GET", &[("Host", host)]), PORT, never);
            assert_eq!(
                refused,
                Err(Refusal::NotLocal {
                    loopback_host: false,
                    proxied: Vec::new()
                }),
                "{host}"
            );
        }
        assert!(check(&asked("GET", &[]), PORT, never).is_err());
    }

    /// A request through Serve from a device with no user carries no login: refused whether Serve
    /// keeps the original `Host` or passes the loopback's.
    #[test]
    fn a_proxied_request_without_a_login_is_refused_whichever_host_it_names() {
        for host in ["box.example.ts.net:9443", "127.0.0.1:47831"] {
            let proxied = asked(
                "GET",
                &[
                    ("Host", host),
                    ("X-Forwarded-Host", "box.example.ts.net:9443"),
                    ("X-Forwarded-For", "100.64.0.9"),
                ],
            );
            let refused = check(&proxied, PORT, never).unwrap_err();
            assert!(
                matches!(&refused, Refusal::NotLocal { proxied, .. } if proxied.len() == 2),
                "{host}: {refused:?}"
            );
        }
        // Serve's other identity headers, or a Funnel's mark, without the login.
        let funnel = asked(
            "GET",
            &[
                ("Host", "127.0.0.1:47831"),
                ("Tailscale-Funnel-Request", "?1"),
            ],
        );
        assert!(check(&funnel, PORT, never).is_err());
    }

    #[test]
    fn the_owners_login_passes_and_any_other_is_refused() {
        // Serve keeping the original Host, or passing the loopback's: the login decides.
        for host in ["box.example.ts.net:9443", "127.0.0.1:47831"] {
            let through = asked(
                "GET",
                &[
                    ("Host", host),
                    ("X-Forwarded-Host", "box.example.ts.net:9443"),
                    ("Tailscale-User-Login", "Owner@Example.com"),
                ],
            );
            assert_eq!(
                check(&through, PORT, owner),
                Ok(Caller::Owner("Owner@Example.com".to_owned())),
                "ASCII case aside"
            );
        }
        let stranger = asked(
            "GET",
            &[
                ("Host", "box.example.ts.net:9443"),
                ("Tailscale-User-Login", "someone@example.org"),
            ],
        );
        assert_eq!(check(&stranger, PORT, owner), Err(Refusal::NotTheOwner));
        let empty = asked(
            "GET",
            &[
                ("Host", "box.example.ts.net:9443"),
                ("Tailscale-User-Login", " "),
            ],
        );
        assert_eq!(check(&empty, PORT, owner), Err(Refusal::NotTheOwner));
        let through = asked(
            "GET",
            &[
                ("Host", "box.example.ts.net:9443"),
                ("Tailscale-User-Login", OWNER),
            ],
        );
        assert_eq!(
            check(&through, PORT, || Ok(Owner::default())),
            Err(Refusal::NoOwner)
        );
        assert_eq!(
            check(&through, PORT, || Err("unknown field".to_owned())),
            Err(Refusal::RegistryUnread)
        );
    }

    /// While the registry's `remote_view` names this machine (EP-31a), another tailnet machine's
    /// name is refused with the owner's login; the machine's own passes with or without its port,
    /// and so does the loopback, the other way Serve may pass `Host`.
    #[test]
    fn remote_view_narrows_the_host_to_this_machines_own_name() {
        let narrowed = || {
            Ok(Owner {
                login: Some(OWNER.to_owned()),
                machine: Some("box.example.ts.net".to_owned()),
            })
        };
        let through = |host: &str| {
            asked(
                "GET",
                &[
                    ("Host", host),
                    ("X-Forwarded-Host", "box.example.ts.net:9443"),
                    ("Tailscale-User-Login", OWNER),
                ],
            )
        };
        for host in [
            "box.example.ts.net:9443",
            "BOX.example.ts.net",
            "127.0.0.1:47831",
        ] {
            assert_eq!(
                check(&through(host), PORT, narrowed),
                Ok(Caller::Owner(OWNER.to_owned())),
                "{host}"
            );
        }
        for host in ["other.example.ts.net:9443", "box.another.ts.net"] {
            assert_eq!(
                check(&through(host), PORT, narrowed),
                Err(Refusal::OtherMachine),
                "{host}"
            );
            // Without the record, any tailnet name passes, as before EP-31a.
            assert!(check(&through(host), PORT, owner).is_ok(), "{host}");
        }
        assert_eq!(
            Refusal::OtherMachine.to_string(),
            "a login with another tailnet machine's name in Host than this one's"
        );
        // Serve passing the loopback's `Host`: the forwarded host names the machine, and is
        // narrowed the same way (the review's finding); with no forwarded host, as before.
        let behind_loopback = |forwarded: Option<&str>| {
            let mut headers = vec![("Host", "127.0.0.1:47831"), ("Tailscale-User-Login", OWNER)];
            headers.extend(forwarded.map(|host| ("X-Forwarded-Host", host)));
            asked("GET", &headers)
        };
        assert_eq!(
            check(
                &behind_loopback(Some("other.example.ts.net:9443")),
                PORT,
                narrowed
            ),
            Err(Refusal::OtherMachine)
        );
        assert!(
            check(
                &behind_loopback(Some("box.example.ts.net:9443")),
                PORT,
                narrowed
            )
            .is_ok()
        );
        assert!(check(&behind_loopback(None), PORT, narrowed).is_ok());
    }

    #[test]
    fn a_machine_name_is_compared_without_its_port_case_or_final_dot() {
        assert!(names_machine(
            "box.example.ts.net:9443",
            "box.example.ts.net"
        ));
        assert!(names_machine("Box.Example.TS.NET", "box.example.ts.net."));
        assert!(!names_machine(
            "box.example.ts.net.evil.example",
            "box.example.ts.net"
        ));
        assert!(!names_machine("box.example.ts.net", ""));
        assert_eq!(
            check_machine(" box.example.ts.net. ").as_deref(),
            Ok("box.example.ts.net")
        );
        for wrong in [
            "box.example.ts.net:9443",
            "example.ts.net",
            "box.example.org",
            "",
        ] {
            assert!(check_machine(wrong).is_err(), "{wrong:?}");
        }
    }

    /// The first request through Serve says how Serve passed `Host` and which proxy headers came,
    /// never a value beyond the login (EP-31a).
    #[test]
    fn the_arrival_line_says_how_serve_passed_the_host() {
        let caller = Caller::Owner(OWNER.to_owned());
        let through = |host: &str| {
            Asked::new(
                "GET",
                "/m/",
                [
                    ("Host", host),
                    ("X-Forwarded-Host", "box.example.ts.net:9443"),
                    ("X-Forwarded-For", "100.64.0.9"),
                    ("X-Forwarded-Proto", "https"),
                    ("Tailscale-User-Login", OWNER),
                    ("Tailscale-User-Name", "Owner"),
                ],
            )
        };
        let machine = Some("box.example.ts.net");
        let named = arrival_line(&caller, &through("box.example.ts.net:9443"), PORT, machine);
        assert_eq!(
            named,
            "owner@example.com · GET /m/ · allowed, the first request through Serve since the \
             window started: Host this machine's tailnet name with a port, proxy headers \
             x-forwarded-host, x-forwarded-for, x-forwarded-proto, tailscale-user-name"
        );
        let loopback = arrival_line(&caller, &through("127.0.0.1:47831"), PORT, machine);
        assert!(
            loopback.contains("Host the loopback with a port"),
            "{loopback}"
        );
        let other = arrival_line(&caller, &through("box.example.ts.net"), PORT, None);
        assert!(
            other.contains("Host a tailnet name with no port"),
            "{other}"
        );
        for line in [named, loopback, other] {
            assert!(
                !line.contains("box.example")
                    && !line.contains("100.64")
                    && !line.contains("Owner"),
                "no header's value beyond the login: {line}"
            );
        }
    }

    #[test]
    fn an_origin_must_be_the_pages_own_and_a_form_must_name_it() {
        let local = |method: &str, origin: Option<&str>| {
            let mut headers = vec![("Host", "127.0.0.1:47831")];
            headers.extend(origin.map(|origin| ("Origin", origin)));
            check(&asked(method, &headers), PORT, never)
        };
        assert_eq!(local("GET", None), Ok(Caller::Local));
        assert_eq!(
            local("POST", Some("http://127.0.0.1:47831")),
            Ok(Caller::Local)
        );
        assert_eq!(
            local("POST", Some("HTTP://127.0.0.1:47831")),
            Ok(Caller::Local)
        );
        assert_eq!(local("POST", None), Err(Refusal::NoOrigin));
        for foreign in [
            "http://evil.example",
            "http://localhost:47831",
            "https://127.0.0.1:47831",
            "null",
        ] {
            assert_eq!(
                local("GET", Some(foreign)),
                Err(Refusal::ForeignOrigin),
                "{foreign}"
            );
            assert_eq!(
                local("POST", Some(foreign)),
                Err(Refusal::ForeignOrigin),
                "{foreign}"
            );
        }
        // Through Serve: the tailnet Host, else the forwarded host behind Serve's loopback Host,
        // under https unless forwarded otherwise.
        let through = |headers: &[(&str, &str)]| {
            let mut all = vec![("Tailscale-User-Login", OWNER)];
            all.extend_from_slice(headers);
            check(&asked("POST", &all), PORT, owner)
        };
        let own = ("Origin", "https://box.example.ts.net:9443");
        assert!(
            through(&[
                ("Host", "127.0.0.1:47831"),
                ("X-Forwarded-Host", "box.example.ts.net:9443"),
                own
            ])
            .is_ok()
        );
        assert!(through(&[("Host", "box.example.ts.net:9443"), own]).is_ok());
        assert_eq!(
            through(&[("Host", "127.0.0.1:47831"), own]),
            Err(Refusal::ForeignOrigin),
            "no forwarded host: the loopback Host is not the page's"
        );
        assert_eq!(
            through(&[
                ("Host", "box.example.ts.net:9443"),
                ("X-Forwarded-Proto", "http"),
                own
            ]),
            Err(Refusal::ForeignOrigin)
        );
        assert_eq!(
            through(&[("Host", "box.example.ts.net:9443")]),
            Err(Refusal::NoOrigin)
        );
    }

    #[test]
    fn the_login_setting_is_one_word() {
        assert_eq!(check_login("  owner@example.com "), Ok("owner@example.com"));
        assert_eq!(check_login("name@github"), Ok("name@github"));
        for wrong in ["", "  ", "two words", "a\tb", "bell\u{7}"] {
            assert!(check_login(wrong).is_err(), "{wrong:?}");
        }
        assert!(check_login(&"x".repeat(LOGIN_CHARS + 1)).is_err());
    }

    #[test]
    fn the_log_names_the_identity_the_request_and_the_reason() {
        let through = Asked::new(
            "POST",
            "/m/p/alpha/tick",
            [
                ("Host", "box.example.ts.net:9443"),
                ("Tailscale-User-Login", OWNER),
            ],
        );
        assert_eq!(
            allowed_line(&Caller::Owner(OWNER.to_owned()), &through),
            "owner@example.com · POST /m/p/alpha/tick · allowed"
        );
        let refused = Refusal::NotLocal {
            loopback_host: false,
            proxied: vec!["x-forwarded-for".to_owned()],
        };
        let anonymous = Asked::new(
            "GET",
            "/m/",
            [
                ("Host", "box.example.ts.net:9443"),
                ("X-Forwarded-For", "100.64.0.9"),
            ],
        );
        let words = refused_line(&anonymous, &refused);
        assert_eq!(
            words,
            "no login · GET /m/ · refused: no login, and not from this machine: Host not the \
             loopback, proxy headers x-forwarded-for"
        );
        assert!(
            !words.contains("box.example") && !words.contains("100.64"),
            "no header's value beyond the login: {words}"
        );
        assert_eq!(
            refused_line(&through, &Refusal::NotTheOwner),
            "owner@example.com · POST /m/p/alpha/tick · refused: through Serve with another \
             login than the owner's"
        );
        assert_eq!(Caller::Local.identity(), "this machine");
        assert!(!Caller::Local.is_remote());
    }

    #[test]
    fn a_refused_identity_is_logged_at_most_once_a_minute() {
        let mut throttle = Throttle::default();
        let start = Instant::now();
        assert!(throttle.due("someone@example.org", start));
        assert!(!throttle.due("Someone@Example.org", start + Duration::from_secs(30)));
        assert!(throttle.due("no login", start + Duration::from_secs(30)));
        assert!(throttle.due("someone@example.org", start + REFUSAL_EVERY));
        // A flood of made-up identities stops at the minute's cap.
        let mut flood = Throttle::default();
        let logged = (0..200)
            .filter(|n| flood.due(&format!("made-up-{n}"), start))
            .count();
        assert_eq!(logged, REFUSED_IDENTITIES);
    }
}
