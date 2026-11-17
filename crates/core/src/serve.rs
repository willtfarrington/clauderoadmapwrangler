//! Tailscale Serve for the phone's path (D-28; EP-31a): the read of Serve's status and the narrow
//! read of this machine's node, the plans of Enable remote view and Disable remote view and their
//! run, and the doctor's "remote view" line. Enable mounts Serve's HTTPS port [`PORT`] at the path
//! [`MOUNT`] on the window's own `/m` on the loopback, and nothing else of the window; Disable
//! takes that mount off. The node's other Serve entries are left alone, and the tool never resets
//! Serve's configuration. A plan is shown whole before it runs, as the hosting actions' are
//! ([`crate::actions`]), and runs through the same [`Runner`], each child in its turn; a run writes
//! the registry's `owner_login` and `remote_view` at the owner's click (rule 13) and keeps an
//! `event` row of kind [`crate::remote::EVENT_KIND`] with the verb, the outcome and the command,
//! never the tailnet name. Tests never run `tailscale`: a fake runner answers (D-29).

use std::path::{Path, PathBuf};
use std::time::Duration;

use chrono::Utc;

use crate::actions::{self, Command, End, Runner};
use crate::doctor::Level;
use crate::ledger::Ledger;
use crate::registry::Store;
use crate::{process, remote};

/// Serve's HTTPS port for the phone's path (D-28).
pub const PORT: u16 = 9443;
/// The phone's path, the only one mounted.
pub const MOUNT: &str = "/m";
/// How long a Serve change may take, holding the children's turn.
pub const TIMEOUT: Duration = Duration::from_secs(60);
/// The doctor's line's name, in the Network group (D-29).
pub const LINE: &str = "remote view";

/// Why a mount on [`PORT`] that is not the phone's refuses Enable.
const TO_ITSELF: &str = "the phone's path needs port 9443 to itself";

/// Enable remote view or Disable remote view.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Verb {
    Enable,
    Disable,
}

impl Verb {
    /// Its word in the event row.
    pub fn id(self) -> &'static str {
        match self {
            Verb::Enable => "enable",
            Verb::Disable => "disable",
        }
    }

    /// The tray's and the Diagnostics pane's words; each opens the dialog.
    pub fn label(self) -> &'static str {
        match self {
            Verb::Enable => "Enable remote view…",
            Verb::Disable => "Disable remote view…",
        }
    }

    /// The dialog's question.
    pub fn question(self) -> &'static str {
        match self {
            Verb::Enable => "Enable remote view?",
            Verb::Disable => "Disable remote view?",
        }
    }

    /// The dialog's button that runs the plan shown.
    pub fn button(self) -> &'static str {
        match self {
            Verb::Enable => "Enable",
            Verb::Disable => "Disable",
        }
    }

    /// What it did.
    pub fn done(self) -> &'static str {
        match self {
            Verb::Enable => "remote view enabled",
            Verb::Disable => "remote view disabled",
        }
    }

    /// The word of "Remote view was not enabled".
    fn past(self) -> &'static str {
        match self {
            Verb::Enable => "enabled",
            Verb::Disable => "disabled",
        }
    }
}

/// Tailscale's command-line client: `tailscale.exe` on PATH, else where its installer puts it,
/// else its bare name.
pub fn program() -> PathBuf {
    crate::doctor::on_path("tailscale.exe")
        .or_else(|| {
            std::env::var_os("ProgramFiles")
                .map(|dir| PathBuf::from(dir).join("Tailscale").join("tailscale.exe"))
                .filter(|candidate| candidate.is_file())
        })
        .unwrap_or_else(|| PathBuf::from("tailscale"))
}

/// This machine as `tailscale status --json` gives it, read narrowly: nothing of the peers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Node {
    /// `BackendState`: `Running`, `Stopped`, `NeedsLogin`…
    pub backend: String,
    /// The login of this machine's user (`User[Self.UserID].LoginName`).
    pub login: Option<String>,
    /// This machine's tailnet name (`Self.DNSName`) without its final dot.
    pub name: Option<String>,
    /// Whether this machine is a tagged device (`Self.Tags`), whose user is no person.
    pub tagged: bool,
    /// Whether the tailnet gives this machine an HTTPS certificate (`CertDomains`): `None` when
    /// the status does not say, `Some(false)` when it names no domain.
    pub certificates: Option<bool>,
}

/// Reads `tailscale status --json`'s backend state, this machine's user's login, its name,
/// whether it is tagged and whether it has a certificate domain, and nothing else.
pub fn parse_node(json: &str) -> Result<Node, String> {
    let value: serde_json::Value =
        serde_json::from_str(json).map_err(|e| format!("tailscale printed no status: {e}"))?;
    let backend = value["BackendState"]
        .as_str()
        .map(str::trim)
        .filter(|state| !state.is_empty())
        .ok_or("tailscale's status names no BackendState")?
        .to_owned();
    let me = &value["Self"];
    let name = me["DNSName"]
        .as_str()
        .map(|name| name.trim().trim_end_matches('.'))
        .filter(|name| !name.is_empty())
        .map(str::to_owned);
    let certificates = match value.get("CertDomains") {
        None => None,
        Some(serde_json::Value::Null) => Some(false),
        Some(serde_json::Value::Array(domains)) => Some(!domains.is_empty()),
        Some(_) => None,
    };
    Ok(Node {
        backend,
        login: user_login(&value, &me["UserID"]),
        name,
        tagged: me["Tags"].as_array().is_some_and(|tags| !tags.is_empty()),
        certificates,
    })
}

/// The login of the user `id` names in the status's `User` map, which is keyed by the id as a
/// string.
fn user_login(status: &serde_json::Value, id: &serde_json::Value) -> Option<String> {
    let key = match id {
        serde_json::Value::Number(number) => number.to_string(),
        serde_json::Value::String(text) => text.trim().to_owned(),
        _ => return None,
    };
    let users = status["User"].as_object()?;
    users
        .get(&key)
        .or_else(|| users.values().find(|user| user["ID"] == *id))
        .and_then(|user| user["LoginName"].as_str())
        .map(str::trim)
        .filter(|login| !login.is_empty())
        .map(str::to_owned)
}

/// What a TCP port of Serve does.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Kind {
    /// HTTPS, whose paths the web mounts serve.
    Https,
    /// Plain HTTP.
    Http,
    /// A TCP forward to `target` (`host:port`), TLS-terminated by Serve when `terminated`.
    Forward { target: String, terminated: bool },
}

/// One TCP port of Serve.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Port {
    pub port: u16,
    pub kind: Kind,
}

/// One web mount of Serve: a path of a host's port, and what serves it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Mount {
    pub port: u16,
    /// The host part of the mount's key: the tailnet name it answers.
    pub host: String,
    /// The mount's path, such as `/m` or `/`.
    pub path: String,
    /// What it proxies to; `None` for a mount that serves files or text.
    pub proxy: Option<String>,
}

/// Serve's configuration as `tailscale serve status --json` gives it.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Status {
    /// The TCP ports, the foreground sessions' with them, by port.
    pub ports: Vec<Port>,
    /// The web mounts, the foreground sessions' with them, by port and path.
    pub mounts: Vec<Mount>,
    /// Whether the node serves a Tailscale service (`Services`).
    pub services: bool,
    /// Whether Funnel is on for any entry (`AllowFunnel`).
    pub funnel: bool,
    /// The ports Funnel is on for, by its `AllowFunnel` keys (`<host>:<port>`): Funnel on the
    /// phone's port would put the pages on the internet (EP-31a's review).
    pub funnel_ports: Vec<u16>,
}

/// Reads `tailscale serve status --json`: the TCP ports and the web mounts, the foreground
/// sessions' merged in, and whether a service or Funnel is on. Unknown keys are ignored, and a TCP
/// key that is no port number is skipped. A node that never had a Serve entry prints `null`, read
/// as an empty configuration.
pub fn parse_status(json: &str) -> Result<Status, String> {
    let text = json.trim();
    if text == "null" {
        return Ok(Status::default());
    }
    let value: serde_json::Value = serde_json::from_str(text)
        .map_err(|e| format!("tailscale serve printed no configuration: {e}"))?;
    if !value.is_object() {
        return Err("tailscale serve's status is not a JSON object".to_owned());
    }
    let mut status = Status::default();
    gather(&value, &mut status);
    if let Some(sessions) = value["Foreground"].as_object() {
        for config in sessions.values() {
            gather(config, &mut status);
        }
    }
    status.ports.sort_by_key(|port| port.port);
    status.ports.dedup();
    status
        .mounts
        .sort_by(|a, b| (a.port, &a.path, &a.host).cmp(&(b.port, &b.path, &b.host)));
    status.mounts.dedup();
    status.funnel_ports.sort_unstable();
    Ok(status)
}

/// Adds one configuration's TCP ports, web mounts and flags to `status`.
fn gather(config: &serde_json::Value, status: &mut Status) {
    let text = |value: &serde_json::Value| {
        value
            .as_str()
            .map(str::trim)
            .filter(|text| !text.is_empty())
            .map(str::to_owned)
    };
    if let Some(ports) = config["TCP"].as_object() {
        for (key, handler) in ports {
            let Ok(port) = key.trim().parse::<u16>() else {
                continue;
            };
            let kind = if let Some(target) = text(&handler["TCPForward"]) {
                Kind::Forward {
                    target,
                    terminated: text(&handler["TerminateTLS"]).is_some(),
                }
            } else if handler["HTTPS"].as_bool() == Some(true) {
                Kind::Https
            } else if handler["HTTP"].as_bool() == Some(true) {
                Kind::Http
            } else {
                continue;
            };
            status.ports.push(Port { port, kind });
        }
    }
    if let Some(sites) = config["Web"].as_object() {
        for (key, site) in sites {
            let Some((host, port)) = key.rsplit_once(':') else {
                continue;
            };
            let (Ok(port), Some(handlers)) = (port.parse::<u16>(), site["Handlers"].as_object())
            else {
                continue;
            };
            for (path, handler) in handlers {
                status.mounts.push(Mount {
                    port,
                    host: host.to_owned(),
                    path: path.clone(),
                    proxy: text(&handler["Proxy"]),
                });
            }
        }
    }
    status.services |= config["Services"]
        .as_object()
        .is_some_and(|services| !services.is_empty());
    if let Some(hosts) = config["AllowFunnel"].as_object() {
        for (key, on) in hosts {
            if on.as_bool() != Some(true) {
                continue;
            }
            status.funnel = true;
            if let Some(port) = key
                .rsplit_once(':')
                .and_then(|(_, port)| port.parse::<u16>().ok())
                && !status.funnel_ports.contains(&port)
            {
                status.funnel_ports.push(port);
            }
        }
    }
}

/// Whether `path` is the phone's: `/m` or `/m/`.
fn is_phone_path(path: &str) -> bool {
    path == MOUNT || path.strip_prefix(MOUNT) == Some("/")
}

/// A proxy's or a forward's target, read: its scheme (empty when it names none), its host in lower
/// case without brackets, its port (the scheme's own when it names none) and its path.
struct Target {
    scheme: String,
    host: String,
    port: Option<u16>,
    path: String,
}

impl Target {
    /// Reads `http://127.0.0.1:3000/path`, `https+insecure://localhost:8443`, `127.0.0.1:3000`,
    /// `[::1]:3000` or a bare port, which Serve takes as the loopback's.
    fn read(text: &str) -> Option<Self> {
        let text = text.trim();
        let (scheme, rest) = text.split_once("://").unwrap_or(("", text));
        let (authority, path) = rest.find('/').map_or((rest, ""), |at| rest.split_at(at));
        let (host, port) = if let Some(inner) = authority.strip_prefix('[') {
            let (host, after) = inner.split_once(']')?;
            (host, after.strip_prefix(':'))
        } else if scheme.is_empty()
            && !authority.is_empty()
            && authority.bytes().all(|b| b.is_ascii_digit())
        {
            ("127.0.0.1", Some(authority))
        } else {
            match authority.rsplit_once(':') {
                Some((host, port)) => (host, Some(port)),
                None => (authority, None),
            }
        };
        let scheme = scheme.to_ascii_lowercase();
        let port = match port {
            Some(port) => Some(port.parse::<u16>().ok()?),
            None => match scheme.as_str() {
                "http" => Some(80),
                "https" | "https+insecure" => Some(443),
                _ => None,
            },
        };
        Some(Self {
            scheme,
            host: host.to_ascii_lowercase(),
            port,
            path: path.to_owned(),
        })
    }

    fn on_loopback(&self) -> bool {
        matches!(self.host.as_str(), "127.0.0.1" | "localhost" | "::1")
    }

    /// Whether it reaches the window's server at `window_port` on the loopback.
    fn reaches(&self, window_port: u16) -> bool {
        self.on_loopback() && self.port == Some(window_port)
    }

    /// Whether it is the target Enable gives Serve: the window's `/m` over plain HTTP at 127.0.0.1
    /// or localhost.
    fn is_phone_pages(&self, window_port: u16) -> bool {
        matches!(self.scheme.as_str(), "" | "http")
            && matches!(self.host.as_str(), "127.0.0.1" | "localhost")
            && self.port == Some(window_port)
            && is_phone_path(&self.path)
    }

    /// The port of a target shaped as Enable gives Serve, whatever the port: the window's `/m`
    /// over plain HTTP at 127.0.0.1 or localhost. Not the window's port, it is an earlier
    /// window's, before its port changed (EP-31a's review).
    fn earlier_phone_pages(&self) -> Option<u16> {
        let shaped = matches!(self.scheme.as_str(), "" | "http")
            && matches!(self.host.as_str(), "127.0.0.1" | "localhost")
            && is_phone_path(&self.path);
        shaped.then_some(self.port).flatten()
    }
}

/// A target in words with no host name: the window's port, a port of this machine, or another
/// machine's.
fn target_words(text: &str, window_port: u16) -> String {
    match Target::read(text) {
        None => "a target that does not read".to_owned(),
        Some(target) if target.reaches(window_port) => format!("the window's port {window_port}"),
        Some(target) => match (target.on_loopback(), target.port) {
            (true, Some(port)) => format!("port {port} of this machine"),
            (true, None) => "this machine".to_owned(),
            (false, Some(port)) => format!("port {port} of another machine"),
            (false, None) => "another machine".to_owned(),
        },
    }
}

/// How Serve stands toward the phone's path and the window's port, each sentence with no host
/// name in it.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Standing {
    /// The host of the phone's mount when Serve has it as Enable makes it: TCP 9443 HTTPS, and a
    /// mount on 9443 at `/m` (or `/m/`) whose proxy points at the window's port with the path
    /// `/m` (or `/m/`), host 127.0.0.1 or localhost, scheme http (or none).
    pub ours: Option<String>,
    /// What holds port 9443 or the path `/m` otherwise, one sentence each: 9443 forwarding raw or
    /// TLS-terminated TCP, 9443 serving plain HTTP, another mount on 9443, or `/m` on 9443 going
    /// elsewhere.
    pub held: Vec<String>,
    /// A forward (raw or TLS-terminated) on any port whose target is the window's port, one
    /// sentence each: it passes raw requests with no login and no proxy header (EP-30's review).
    pub forwards: Vec<String>,
    /// Any other mount whose proxy points at the window's port, one sentence each naming its port
    /// and path: it puts the window's other routes on the tailnet, or the phone's pages beside the
    /// mount Disable takes off.
    pub elsewhere: Vec<String>,
    /// A mount shaped as Enable makes it but to another port of this machine, by its host and
    /// that port: an earlier window's, before its port changed (EP-31a's review), when the
    /// registry's record names its host ([`Standing::earlier_of`]); it holds the port either way.
    pub earlier: Option<(String, u16)>,
    /// Funnel is on for port 9443, which would put the phone's pages on the internet (EP-31a's
    /// review; D-28: tailnet only, never Funnel).
    pub funnel: bool,
}

impl Standing {
    /// The port of the mount an earlier window made, when the registry's record `machine` names
    /// its host: the tool's own, which Disable may take off. Without the record, a mount to
    /// another port at the phone's path may be another service's, and is left alone.
    pub fn earlier_of(&self, machine: Option<&str>) -> Option<u16> {
        let (host, port) = self.earlier.as_ref()?;
        machine
            .is_some_and(|machine| remote::names_machine(host, machine))
            .then_some(*port)
    }
}

impl Status {
    /// How Serve stands toward the phone's path and `window_port`, the window's port.
    pub fn standing(&self, window_port: u16) -> Standing {
        let target = |mount: &Mount| mount.proxy.as_deref().and_then(Target::read);
        // The mount's key exactly as Enable writes it, which Disable's `off` names (the review's:
        // a `/m/` key is no mount `--set-path=/m off` reaches).
        let shaped = |mount: &Mount| {
            mount.port == PORT
                && mount.path == MOUNT
                && target(mount).is_some_and(|target| target.is_phone_pages(window_port))
        };
        let https = self
            .ports
            .iter()
            .any(|port| port.port == PORT && port.kind == Kind::Https);
        let mut standing = Standing {
            ours: self
                .mounts
                .iter()
                .find(|mount| https && shaped(mount))
                .map(|mount| mount.host.clone()),
            funnel: self.funnel_ports.contains(&PORT),
            ..Standing::default()
        };
        for port in &self.ports {
            let (target, how) = match &port.kind {
                Kind::Forward {
                    target,
                    terminated: false,
                } => (target, "forwards raw TCP"),
                Kind::Forward {
                    target,
                    terminated: true,
                } => (target, "terminates TLS and forwards TCP"),
                Kind::Http if port.port == PORT => {
                    standing.held.push(format!("port {PORT} serves plain HTTP"));
                    continue;
                }
                Kind::Http | Kind::Https => continue,
            };
            if Target::read(target).is_some_and(|target| target.reaches(window_port)) {
                standing.forwards.push(format!(
                    "port {} {how} to the window's port {window_port}: raw requests reach the \
                     window with no login and no proxy header",
                    port.port
                ));
            } else if port.port == PORT {
                standing.held.push(format!(
                    "port {PORT} {how} to {}",
                    target_words(target, window_port)
                ));
            }
        }
        for mount in self.mounts.iter().filter(|mount| !shaped(mount)) {
            let (port, path) = (mount.port, &mount.path);
            if let Some(target) = target(mount).filter(|target| target.reaches(window_port)) {
                standing.elsewhere.push(if is_phone_path(&target.path) {
                    format!(
                        "port {port}'s path {path} also proxies to the window's phone pages, a \
                         mount Disable leaves alone"
                    )
                } else {
                    format!(
                        "port {port}'s path {path} proxies to the window's port {window_port}, \
                         which puts the window's other routes on the tailnet"
                    )
                });
                continue;
            }
            if port != PORT {
                continue;
            }
            // Shaped as the phone's mount, to another port of this machine: an earlier window's
            // perhaps, which only the registry's record can tell from another service's, so it
            // holds the port all the same (the review's).
            if let Some(earlier) = target(mount).and_then(|target| target.earlier_phone_pages())
                && path == MOUNT
            {
                standing.earlier = Some((mount.host.clone(), earlier));
            }
            standing
                .held
                .push(match (mount.proxy.as_deref(), is_phone_path(path)) {
                    (Some(proxy), true) => format!(
                        "port {PORT}'s path {path} proxies to {}, not the window's phone pages",
                        target_words(proxy, window_port)
                    ),
                    (None, true) => format!(
                        "port {PORT}'s path {path} serves files or text, not the window's phone \
                         pages"
                    ),
                    (Some(proxy), false) => format!(
                        "port {PORT} serves another path, {path}, for another service: it \
                         proxies to {}",
                        target_words(proxy, window_port)
                    ),
                    (None, false) => {
                        format!("port {PORT} serves files or text at another path, {path}")
                    }
                });
        }
        standing
    }
}

/// The remote view's URL on `name`, this machine's tailnet name: the phone's pages.
pub fn url(name: &str) -> String {
    link(name, &format!("{MOUNT}/"))
}

/// A page's link on `name`, its `path` starting with `/`: a push's click (EP-31a).
pub fn link(name: &str, path: &str) -> String {
    let name = name.trim().trim_end_matches('.');
    let slash = if path.starts_with('/') { "" } else { "/" };
    format!("https://{name}:{PORT}{slash}{path}")
}

/// What a plan is read from: the verb, the window's port and whether its server listens there,
/// the registry's record now, the window's own refusals, and Tailscale's client.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Facts {
    pub verb: Verb,
    /// The window's port.
    pub port: u16,
    /// Whether the window's server listens on it.
    pub listening: bool,
    /// The registry's `owner_login` now.
    pub login: Option<String>,
    /// The registry's `remote_view` now.
    pub machine: Option<String>,
    /// The window's own refusals (a screenshot run).
    pub refusals: Vec<String>,
    /// Tailscale's client ([`program`]).
    pub program: PathBuf,
}

/// Enable or Disable, planned: what the dialog shows before it runs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Plan {
    pub verb: Verb,
    /// The window's port, against which Serve is read again after the command (EP-31a's review).
    pub port: u16,
    /// `None` when there is nothing to run: a refusal, the mount there already for Enable, or no
    /// mount for Disable.
    pub command: Option<Command>,
    /// Enable: the login read from the node, written at the click; Disable: `None`.
    pub login: Option<String>,
    /// Enable: the node's name, written at the click; Disable: the registry's record it takes
    /// out.
    pub machine: Option<String>,
    /// The remote view's URL: Enable's from the node's name, Disable's from the registry's record
    /// or Serve's mount.
    pub url: Option<String>,
    /// What it does, a sentence each; none when refused.
    pub words: Vec<String>,
    pub warnings: Vec<String>,
    /// What refuses it; the owner can only close the dialog then.
    pub problems: Vec<String>,
}

impl Plan {
    pub fn is_refused(&self) -> bool {
        !self.problems.is_empty()
    }
}

/// `tailscale` with `args`, given `timeout`.
fn tailscale(program: &Path, args: &[&str], timeout: Duration) -> Command {
    Command {
        program: program.to_owned(),
        args: args.iter().map(|arg| (*arg).to_owned()).collect(),
        long: false,
        timeout,
        folder: None,
    }
}

/// The plan of `facts`' verb, read through `runner`: Enable asks `tailscale status --json` and
/// then `tailscale serve status --json`, Disable the second alone. A refused plan runs nothing and
/// says only why.
pub fn plan(facts: &Facts, runner: &dyn Runner) -> Plan {
    let mut plan = Plan {
        verb: facts.verb,
        port: facts.port,
        command: None,
        login: None,
        machine: None,
        url: None,
        words: Vec::new(),
        warnings: Vec::new(),
        problems: facts.refusals.clone(),
    };
    match facts.verb {
        Verb::Enable => enable(facts, runner, &mut plan),
        Verb::Disable => disable(facts, runner, &mut plan),
    }
    if plan.is_refused() {
        plan.command = None;
        plan.words.clear();
    }
    plan
}

/// This machine's node, read through `runner`: `tailscale status` exits with an error while
/// stopped or logged out, and may still print its JSON.
fn node(facts: &Facts, runner: &dyn Runner) -> Result<Node, String> {
    let read = tailscale(&facts.program, &["status", "--json"], process::TIMEOUT);
    let exit = runner.run(&read)?;
    parse_node(&exit.stdout).map_err(|why| match exit.code {
        Some(0) => why,
        _ => actions::said("tailscale status", &exit),
    })
}

/// Serve's configuration, read through `runner`; a failure in words for a plan's problem.
fn serve_status(facts: &Facts, runner: &dyn Runner) -> Result<Status, String> {
    let read = tailscale(
        &facts.program,
        &["serve", "status", "--json"],
        process::TIMEOUT,
    );
    let unread = |why: String| format!("Serve's configuration did not read: {why}");
    let exit = runner.run(&read).map_err(unread)?;
    if exit.code != Some(0) {
        return Err(actions::said("tailscale serve status", &exit));
    }
    parse_status(&exit.stdout).map_err(unread)
}

fn enable(facts: &Facts, runner: &dyn Runner, plan: &mut Plan) {
    let port = facts.port;
    if !facts.listening {
        plan.problems.push(format!(
            "the window's server does not listen on port {port}, so Serve would proxy to nothing: \
             the status bar says why"
        ));
    }
    let node = match node(facts, runner) {
        Ok(node) => node,
        Err(why) => {
            plan.problems
                .push(format!("Tailscale's status did not read: {why}"));
            return;
        }
    };
    if node.backend != "Running" {
        plan.problems.push(format!(
            "Tailscale is {}: the tailnet is not up",
            node.backend
        ));
    }
    if node.tagged {
        plan.problems.push(
            "this machine is a tagged device, whose user is no person: Enable reads the owner's \
             login from this machine's user, so set it by hand with crw setting owner_login"
                .to_owned(),
        );
    }
    match node.login.as_deref() {
        None => plan
            .problems
            .push("Tailscale's status names no login for this machine's user".to_owned()),
        Some(login) if remote::check_login(login).is_err() => plan.problems.push(
            "the login Tailscale's status names for this machine's user is not the one word \
             owner_login takes"
                .to_owned(),
        ),
        Some(_) => {}
    }
    match node.name.as_deref() {
        None => plan.problems.push(
            "Tailscale's status names no DNS name for this machine: MagicDNS may be off".to_owned(),
        ),
        Some(name) if remote::check_machine(name).is_err() => plan.problems.push(
            "this machine's DNS name is no tailnet name of the form <machine>.<tailnet>.ts.net, \
             which remote_view takes"
                .to_owned(),
        ),
        Some(_) => {}
    }
    if node.certificates == Some(false) {
        plan.problems.push(
            "the tailnet gives this machine no HTTPS certificate (its status names no certificate \
             domain): turn HTTPS certificates on in the tailnet's DNS settings, then Enable again"
                .to_owned(),
        );
    }
    let status = match serve_status(facts, runner) {
        Ok(status) => status,
        Err(why) => {
            plan.problems.push(why);
            return;
        }
    };
    let standing = status.standing(port);
    for sentence in standing.forwards.iter().chain(&standing.elsewhere) {
        plan.problems.push(format!(
            "{sentence}; take it off by hand first: the tool never resets Serve's configuration"
        ));
    }
    for sentence in &standing.held {
        plan.problems.push(format!("{sentence}: {TO_ITSELF}"));
    }
    if let Some(earlier) = standing.earlier_of(facts.machine.as_deref()) {
        plan.problems.push(format!(
            "that mount to port {earlier} is the one an earlier window made, before this \
             window's port became {port}: Disable remote view takes it off, then Enable again"
        ));
    }
    // Funnel would put the phone's pages on the internet (D-28: tailnet only).
    if standing.funnel {
        plan.problems.push(format!(
            "Funnel is on for port {PORT}, which puts what it serves on the internet: the \
             phone's path stays on the tailnet, so turn Funnel off for port {PORT} by hand first"
        ));
    }
    if let (Some(host), Some(name)) = (standing.ours.as_deref(), node.name.as_deref())
        && !remote::names_machine(host, name)
    {
        plan.problems.push(
            "Serve holds the phone's path under another machine name than this machine's (a \
             rename since?), which Tailscale's off cannot reach by this name: look at tailscale \
             serve status"
                .to_owned(),
        );
    }
    plan.login.clone_from(&node.login);
    plan.machine.clone_from(&node.name);
    plan.url = node.name.as_deref().map(url);
    if standing.ours.is_none() {
        let https = format!("--https={PORT}");
        let set_path = format!("--set-path={MOUNT}");
        let target = format!("http://127.0.0.1:{port}{MOUNT}");
        plan.command = Some(tailscale(
            &facts.program,
            &["serve", "--bg", &https, &set_path, &target],
            TIMEOUT,
        ));
    } else {
        plan.words.push(
            "Serve already mounts the phone's path: Enable writes the login and this machine's \
             name to the registry."
                .to_owned(),
        );
    }
    plan.words.push(format!(
        "Serve answers the path {MOUNT}/ on port {PORT} of this machine's tailnet name with the \
         window's phone pages, proxied to http://127.0.0.1:{port}{MOUNT}; the window's other \
         routes stay on this machine."
    ));
    if let Some(login) = &node.login {
        plan.words.push(format!(
            "A request passes only with your login, {login}, which Serve sets for your own \
             devices; other users and tagged devices are refused."
        ));
    }
    plan.words.push(
        "The node's other Serve entries are left alone, and the tool never resets Serve's \
         configuration."
            .to_owned(),
    );
    plan.words
        .push("Disable remote view takes the mount off again.".to_owned());
    if let (Some(held), Some(login)) = (facts.login.as_deref(), node.login.as_deref())
        && !held.trim().eq_ignore_ascii_case(login)
    {
        plan.warnings.push(format!(
            "The registry names another login: Enable replaces it with {login}."
        ));
    }
    if status.funnel && !standing.funnel {
        plan.warnings.push(
            "Funnel is on for another entry of this node; the phone's mount stays on the \
             tailnet, never on Funnel."
                .to_owned(),
        );
    }
    plan.warnings.push(
        "A phone whose other VPN is on may not reach the tailnet, and so not the page.".to_owned(),
    );
}

fn disable(facts: &Facts, runner: &dyn Runner, plan: &mut Plan) {
    plan.machine.clone_from(&facts.machine);
    plan.url = facts.machine.as_deref().map(url);
    let status = match serve_status(facts, runner) {
        Ok(status) => status,
        Err(why) => {
            plan.problems.push(why);
            return;
        }
    };
    let standing = status.standing(facts.port);
    // An earlier window's mount, to the port it listened on then, is the tool's own too when the
    // registry's record names its host: the same `off` takes it off (EP-31a's review). Without
    // the record it may be another service's, and is left alone.
    let mounted = standing.ours.clone().or_else(|| {
        standing
            .earlier_of(facts.machine.as_deref())
            .and(standing.earlier.as_ref().map(|(host, _)| host.clone()))
    });
    match (mounted.as_deref(), facts.machine.as_deref()) {
        (Some(host), machine) => {
            let https = format!("--https={PORT}");
            let set_path = format!("--set-path={MOUNT}");
            plan.command = Some(tailscale(
                &facts.program,
                &["serve", &https, &set_path, "off"],
                TIMEOUT,
            ));
            plan.words.push(
                "Takes the phone's mount off Serve: its link stops answering, and the node's \
                 other entries are left alone."
                    .to_owned(),
            );
            if machine.is_some() {
                plan.words
                    .push("Takes this machine's name out of the registry.".to_owned());
            } else {
                plan.url = Some(url(host));
            }
        }
        (None, Some(_)) => plan.words.push(
            "Serve holds no mount of the phone's path: Disable takes this machine's name out of \
             the registry."
                .to_owned(),
        ),
        (None, None) => plan.problems.push(
            "remote view is off: Serve holds no mount of the phone's path and the registry no \
             record of one"
                .to_owned(),
        ),
    }
    plan.words
        .push("The login stays in the registry for the next Enable.".to_owned());
}

/// What running a plan did.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ran {
    pub verb: Verb,
    /// The command's line; `None` when none ran.
    pub command: Option<String>,
    /// Done, or Failed in words; never Warned.
    pub end: End,
    /// What the command printed on its standard output, trimmed: Serve's own words, which may
    /// hold the URL; never logged.
    pub printed: String,
    /// Why the registry was not written, when it was not.
    pub unwritten: Option<String>,
    /// Why the ledger did not take the row, when it did not.
    pub unlogged: Option<String>,
    /// The plan's URL.
    pub url: Option<String>,
}

impl Ran {
    /// The status bar's words, with no host name or URL in them.
    pub fn words(&self) -> String {
        let ran = self.command.is_some();
        let written = self.unwritten.is_none();
        let words = match (&self.end, self.verb) {
            (End::Failed(why), verb) => format!("Remote view was not {}: {why}", verb.past()),
            (End::Done | End::Warned(_), Verb::Enable) => match (ran, written) {
                (true, true) => {
                    format!("Remote view enabled: Serve mounts the phone's path on port {PORT}")
                }
                (true, false) => format!(
                    "Serve mounts the phone's path on port {PORT}, but remote view is not on \
                     until the registry holds the login and the machine's name (Enable again)"
                ),
                (false, true) => "Remote view enabled: Serve already mounted the phone's path; \
                                  the login and the machine's name are in the registry"
                    .to_owned(),
                (false, false) => {
                    "Remote view was not enabled: Serve already mounted the phone's path".to_owned()
                }
            },
            (End::Done | End::Warned(_), Verb::Disable) => match (ran, written) {
                (true, true) => {
                    "Remote view disabled: Serve's mount of the phone's path is off".to_owned()
                }
                (true, false) => "Remote view disabled: Serve's mount of the phone's path is off, \
                                  but the registry still names this machine (crw setting \
                                  remote_view none takes it out)"
                    .to_owned(),
                (false, true) => {
                    "Remote view disabled: the registry's record is out (Serve held no mount)"
                        .to_owned()
                }
                (false, false) => "Remote view was not disabled: Serve held no mount".to_owned(),
            },
        };
        let words = match &self.unwritten {
            Some(why) => format!("{words}; {why}"),
            None => words,
        };
        masked(&words)
    }

    /// The event row's detail: the verb, the outcome and the command, with no printed text and no
    /// host name.
    pub fn detail(&self) -> String {
        format!(
            "{}: {} · {}",
            self.verb.id(),
            self.words(),
            self.command.as_deref().unwrap_or("no command")
        )
    }
}

/// `text` with each word that names a tailnet machine put as "<this machine's name>": Serve's own
/// words may name it, and the status bar, the doctor's problems and the event row never do.
fn masked(text: &str) -> String {
    text.split_whitespace()
        .map(|word| {
            if word.to_ascii_lowercase().contains(".ts.net") {
                "<this machine's name>"
            } else {
                word
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// Runs a plan the owner confirmed through `runner`; `None` when it is refused. Only when it ended
/// well (or had nothing to run) is the registry written, in one change of `store` at the owner's
/// click: Enable writes the login and this machine's name, Disable takes the name out and keeps
/// the login. Each run, failed or not, is an `event` row in `ledger`.
pub fn run(
    plan: &Plan,
    runner: &dyn Runner,
    store: Option<&Store>,
    ledger: Option<&Path>,
    today: &str,
) -> Option<Ran> {
    if plan.is_refused() {
        return None;
    }
    let (end, printed) = match &plan.command {
        None => (End::Done, String::new()),
        Some(command) => match runner.run(command) {
            Ok(exit) if exit.code == Some(0) => {
                let printed = exit.stdout.trim().to_owned();
                match verified(plan, runner, &command.program) {
                    Ok(()) => (End::Done, printed),
                    Err(why) => (
                        End::Failed(format!(
                            "tailscale serve ended without an error, but {why}, so nothing is \
                             written: its own words are below"
                        )),
                        printed,
                    ),
                }
            }
            Ok(exit) => (
                End::Failed(actions::said("tailscale serve", &exit)),
                exit.stdout.trim().to_owned(),
            ),
            Err(why) => (End::Failed(why), String::new()),
        },
    };
    let mut ran = Ran {
        verb: plan.verb,
        command: plan.command.as_ref().map(Command::line),
        end,
        printed,
        unwritten: None,
        unlogged: None,
        url: plan.url.clone(),
    };
    if ran.end == End::Done
        && let Some(store) = store
    {
        ran.unwritten = write(plan, store, today).err();
    }
    if let Some(path) = ledger {
        ran.unlogged = log(path, &ran).err();
    }
    Some(ran)
}

/// Serve read again after a command that ended well (EP-31a's review): whether it now shows what
/// the command was to do. A prompt answered by the child's empty input, or a feature's flow that
/// does not wait, may end the command well with nothing changed.
fn verified(plan: &Plan, runner: &dyn Runner, program: &Path) -> Result<(), String> {
    let read = tailscale(program, &["serve", "status", "--json"], process::TIMEOUT);
    let exit = runner
        .run(&read)
        .map_err(|why| format!("Serve's configuration did not read again: {why}"))?;
    if exit.code != Some(0) {
        return Err(actions::said("tailscale serve status", &exit));
    }
    let standing = parse_status(&exit.stdout)
        .map_err(|why| format!("Serve's configuration did not read again: {why}"))?
        .standing(plan.port);
    match plan.verb {
        Verb::Enable if standing.ours.is_none() => {
            Err("Serve's configuration, read again, does not show the phone's mount".to_owned())
        }
        Verb::Disable
            if standing.ours.is_some()
                || standing.earlier_of(plan.machine.as_deref()).is_some() =>
        {
            Err("Serve's configuration, read again, still shows the phone's mount".to_owned())
        }
        _ => Ok(()),
    }
}

/// The registry's one change at the owner's click (rule 13).
fn write(plan: &Plan, store: &Store, today: &str) -> Result<(), String> {
    store
        .change(today, |changes| {
            match plan.verb {
                Verb::Enable => {
                    if let Some(login) = &plan.login {
                        changes.set_setting("owner_login", login)?;
                    }
                    if let Some(machine) = &plan.machine {
                        changes.set_setting("remote_view", machine)?;
                    }
                }
                Verb::Disable => changes.set_setting("remote_view", "none")?,
            }
            Ok(())
        })
        .map(|_| ())
        .map_err(|e| format!("the registry was not written: {e}"))
}

/// Keeps `ran` as an `event` row of the remote-action log's kind, under no project.
fn log(path: &Path, ran: &Ran) -> Result<(), String> {
    Ledger::open(path)
        .and_then(|ledger| ledger.add_event(Utc::now(), remote::EVENT_KIND, None, &ran.detail()))
        .map(|_| ())
        .map_err(|e| format!("the ledger did not take the remote view's line: {e}"))
}

/// The doctor's "remote view" line (D-29): Serve's status (`None` when it was not asked, Tailscale
/// not being found) against `machine`, the registry's `remote_view`, `port` being the window's. Its
/// level is the worst of its parts, its words the parts joined by "; ". No part names a host but
/// the URL of a remote view that is on with nothing else to say.
pub fn line(
    status: Option<&Result<Status, String>>,
    machine: Option<&str>,
    port: u16,
) -> (Level, String) {
    let parts = match status {
        None => vec![match machine {
            None => (
                Level::Note,
                "off; Serve's status not read (Tailscale not found)".to_owned(),
            ),
            Some(_) => (
                Level::Warn,
                "on in the registry, but Serve's status was not read: Tailscale not found"
                    .to_owned(),
            ),
        }],
        Some(Err(why)) => {
            let why = masked(why);
            vec![match machine {
                None => (Level::Note, format!("Serve's status did not read: {why}")),
                Some(_) => (
                    Level::Warn,
                    format!("on in the registry, but Serve's status did not read: {why}"),
                ),
            }]
        }
        Some(Ok(status)) => parts(&status.standing(port), machine),
    };
    let level = parts
        .iter()
        .map(|(level, _)| *level)
        .max()
        .unwrap_or(Level::Ok);
    let words = parts
        .into_iter()
        .map(|(_, words)| words)
        .collect::<Vec<_>>()
        .join("; ");
    (level, words)
}

/// The line's parts from Serve's standing and the registry's record: the mount against the record
/// first, then the window's port exposed elsewhere, then what holds the phone's port while the
/// mount is not there.
fn parts(standing: &Standing, machine: Option<&str>) -> Vec<(Level, String)> {
    let mut rest: Vec<(Level, String)> = Vec::new();
    for sentence in &standing.forwards {
        rest.push((Level::Fail, format!("{sentence}; take it off by hand")));
    }
    for sentence in &standing.elsewhere {
        rest.push((Level::Warn, format!("{sentence}; take it off by hand")));
    }
    // D-28: tailnet only, never Funnel (EP-31a's review).
    if standing.funnel {
        rest.push((
            Level::Fail,
            format!(
                "Funnel is on for port {PORT}, which puts what it serves on the internet; turn \
                 it off by hand"
            ),
        ));
    }
    if let Some(earlier) = standing.earlier_of(machine)
        && standing.ours.is_none()
    {
        rest.push((
            Level::Warn,
            format!(
                "Serve mounts the phone's path to port {earlier} of this machine, an earlier \
                 window's: Disable, then Enable again"
            ),
        ));
    }
    if standing.ours.is_none() {
        for sentence in &standing.held {
            rest.push((Level::Note, format!("{sentence}: {TO_ITSELF}")));
        }
    }
    let head = match (standing.ours.as_deref(), machine) {
        (Some(host), Some(machine)) if remote::names_machine(host, machine) => (
            Level::Ok,
            if rest.is_empty() {
                format!(
                    "on at {}: Serve proxies the phone's path alone",
                    url(machine)
                )
            } else {
                "on: Serve proxies the phone's path".to_owned()
            },
        ),
        (Some(_), Some(_)) => (
            Level::Warn,
            "Serve mounts the phone's path under another machine name than the registry's: \
             Disable, then Enable again"
                .to_owned(),
        ),
        (Some(_), None) => (
            Level::Warn,
            "Serve mounts the phone's path, but the registry has no record of it: Enable remote \
             view writes the login and the machine's name"
                .to_owned(),
        ),
        (None, Some(_)) => (
            Level::Warn,
            "on in the registry while Serve holds no mount of the phone's path: Enable again, or \
             Disable to take the record out"
                .to_owned(),
        ),
        (None, None) => (
            Level::Note,
            "off: Enable remote view (the Diagnostics pane or the tray's menu) mounts the phone's \
             path on the tailnet"
                .to_owned(),
        ),
    };
    std::iter::once(head).chain(rest).collect()
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::sync::Mutex;

    use super::*;
    use crate::actions::Exit;
    use crate::files::tests::Scratch;
    use crate::registry::Defaults;

    /// The window's port in these tests.
    const WINDOW: u16 = 47831;
    const OWNER: &str = "owner@example.com";
    const BOX: &str = "box.example.ts.net";
    const TODAY: &str = "2026-11-17";
    /// A tailnet that gives this machine a certificate domain.
    const CERTS: Option<&str> = Some(r#"["box.example.ts.net"]"#);
    /// This machine as `tailscale status --json` gives it: untagged, its user the owner.
    const ME: &str = r#"{"ID":"n1","UserID":4242,"HostName":"box","DNSName":"box.example.ts.net.","Online":true}"#;

    /// The phone's mount as Enable makes it, beside another service on 443.
    const OURS: &str = r#"{"TCP":{"443":{"HTTPS":true},"9443":{"HTTPS":true}},
        "Web":{"box.example.ts.net:443":{"Handlers":{"/":{"Proxy":"http://127.0.0.1:3000"}}},
               "box.example.ts.net:9443":{"Handlers":{"/m":{"Proxy":"http://127.0.0.1:47831/m"}}}}}"#;

    /// What Serve prints when Enable's change goes through, the URL in it.
    const STARTED: &str = "Available within your tailnet:\n\nhttps://box.example.ts.net:9443/m\n\
                           |-- proxy http://127.0.0.1:47831/m\n\nServe started and running in \
                           the background.\n";

    /// `tailscale status --json` with `backend`, `me` as `Self` and `certs` as `CertDomains`
    /// (absent when `None`): the user map keyed by the id as a string, and a peer whose names are
    /// never kept.
    fn node_json(backend: &str, me: &str, certs: Option<&str>) -> String {
        let certs = certs.map_or_else(String::new, |certs| format!(r#""CertDomains":{certs},"#));
        format!(
            r#"{{"Version":"1.0.0","BackendState":"{backend}",{certs}"Self":{me},
            "User":{{"4242":{{"ID":4242,"LoginName":"owner@example.com","DisplayName":"Owner"}},
                     "77":{{"ID":77,"LoginName":"friend@example.org","DisplayName":"Friend"}}}},
            "Peer":{{"nodekey:1":{{"HostName":"phone","DNSName":"phone.example.ts.net.",
                     "UserID":77,"Online":true}}}}}}"#
        )
    }

    /// A running machine with a certificate domain.
    fn running() -> String {
        node_json("Running", ME, CERTS)
    }

    fn ok(stdout: &str) -> Exit {
        Exit {
            code: Some(0),
            stdout: stdout.to_owned(),
            stderr: String::new(),
        }
    }

    fn failed(code: i32, stdout: &str, stderr: &str) -> Exit {
        Exit {
            code: Some(code),
            stdout: stdout.to_owned(),
            stderr: stderr.to_owned(),
        }
    }

    /// A runner that answers `tailscale status`, `tailscale serve status` and any change from its
    /// script and keeps each command it was asked to run: never `tailscale` itself (D-29). Once a
    /// change has run, Serve's status answers `after` when it is set: the configuration as the
    /// change left it, which the run reads again (EP-31a's review).
    struct Fake {
        asked: Mutex<Vec<Command>>,
        node: Result<Exit, String>,
        serve: Result<Exit, String>,
        change: Result<Exit, String>,
        after: Option<Result<Exit, String>>,
        changed: Mutex<bool>,
    }

    impl Fake {
        fn new(node: &str, serve: &str) -> Self {
            Self {
                asked: Mutex::new(Vec::new()),
                node: Ok(ok(node)),
                serve: Ok(ok(serve)),
                change: Ok(ok(STARTED)),
                after: None,
                changed: Mutex::new(false),
            }
        }

        /// The same, Serve's status answering `after` once a change has run.
        fn then(node: &str, serve: &str, after: &str) -> Self {
            Self {
                after: Some(Ok(ok(after))),
                ..Self::new(node, serve)
            }
        }

        /// The lines of the commands asked, in their order.
        fn asked(&self) -> Vec<String> {
            self.asked
                .lock()
                .unwrap()
                .iter()
                .map(Command::line)
                .collect()
        }
    }

    impl Runner for Fake {
        fn run(&self, command: &Command) -> Result<Exit, String> {
            self.asked.lock().unwrap().push(command.clone());
            let args: Vec<&str> = command.args.iter().map(String::as_str).collect();
            let changed = *self.changed.lock().unwrap();
            match args.as_slice() {
                ["status", "--json"] => self.node.clone(),
                ["serve", "status", "--json"] => match (&self.after, changed) {
                    (Some(after), true) => after.clone(),
                    _ => self.serve.clone(),
                },
                _ => {
                    *self.changed.lock().unwrap() = true;
                    self.change.clone()
                }
            }
        }
    }

    fn facts(verb: Verb) -> Facts {
        Facts {
            verb,
            port: WINDOW,
            listening: true,
            login: None,
            machine: None,
            refusals: Vec::new(),
            program: PathBuf::from("tailscale"),
        }
    }

    /// Disable's facts while the registry names this machine.
    fn recorded() -> Facts {
        Facts {
            machine: Some(BOX.to_owned()),
            ..facts(Verb::Disable)
        }
    }

    /// A scratch registry, as the registry's tests make one.
    fn store(scratch: &Scratch) -> Store {
        let data = scratch.0.join("data");
        Store::new(
            scratch.0.join("config").join("registry.toml"),
            scratch.0.join("local").join("backups"),
            Defaults {
                repository: data.join("wrangler"),
                data_root: data,
            },
        )
    }

    fn https(port: u16) -> Port {
        Port {
            port,
            kind: Kind::Https,
        }
    }

    fn forward(port: u16, target: &str, terminated: bool) -> Port {
        Port {
            port,
            kind: Kind::Forward {
                target: target.to_owned(),
                terminated,
            },
        }
    }

    fn mount(port: u16, path: &str, proxy: Option<&str>) -> Mount {
        Mount {
            port,
            host: BOX.to_owned(),
            path: path.to_owned(),
            proxy: proxy.map(str::to_owned),
        }
    }

    fn serving(ports: Vec<Port>, mounts: Vec<Mount>) -> Status {
        Status {
            ports,
            mounts,
            services: false,
            funnel: false,
            funnel_ports: Vec::new(),
        }
    }

    /// The phone's mount as Enable makes it.
    fn ours() -> Status {
        serving(
            vec![https(PORT)],
            vec![mount(PORT, "/m", Some("http://127.0.0.1:47831/m"))],
        )
    }

    /// Every sentence of a standing.
    fn sentences(standing: &Standing) -> Vec<&String> {
        standing
            .held
            .iter()
            .chain(&standing.forwards)
            .chain(&standing.elsewhere)
            .collect()
    }

    #[test]
    fn verbs_word_themselves() {
        assert_eq!(Verb::Enable.id(), "enable");
        assert_eq!(Verb::Disable.id(), "disable");
        assert_eq!(Verb::Enable.label(), "Enable remote view…");
        assert_eq!(Verb::Disable.question(), "Disable remote view?");
        assert_eq!(Verb::Enable.button(), "Enable");
        assert_eq!(Verb::Disable.done(), "remote view disabled");
        // On PATH, where its installer puts it, or its bare name: found, never run.
        assert_eq!(program().file_stem().unwrap(), "tailscale");
    }

    #[test]
    fn the_links_name_the_machine_and_the_port() {
        assert_eq!(url(BOX), "https://box.example.ts.net:9443/m/");
        assert_eq!(
            url("box.example.ts.net."),
            "https://box.example.ts.net:9443/m/"
        );
        assert_eq!(
            link(BOX, "/m/s/3c9d2e71"),
            "https://box.example.ts.net:9443/m/s/3c9d2e71"
        );
        assert_eq!(
            link(BOX, "m/decisions"),
            "https://box.example.ts.net:9443/m/decisions"
        );
    }

    #[test]
    fn reads_an_empty_configuration() {
        assert_eq!(parse_status("{}\n"), Ok(Status::default()));
        assert_eq!(
            parse_status("null\n"),
            Ok(Status::default()),
            "a node that never served"
        );
        assert_eq!(Status::default().standing(WINDOW), Standing::default());
    }

    #[test]
    fn refuses_what_is_not_a_json_object() {
        for text in ["", "not json", "[]", "3", "\"text\""] {
            assert!(parse_status(text).is_err(), "{text:?}");
        }
        assert!(
            parse_status("[]")
                .unwrap_err()
                .contains("not a JSON object")
        );
    }

    /// The node's shape, names synthetic: each kind of port, a mount that proxies and one that
    /// serves text, keys that are no port, unknown keys, and the service and Funnel flags.
    #[test]
    fn reads_the_ports_the_mounts_and_the_flags() {
        let json = r#"{
            "TCP": {
                "443": {"HTTPS": true},
                "80": {"HTTP": true},
                "5000": {"TCPForward": "127.0.0.1:47831"},
                "6000": {"TCPForward": "localhost:7000", "TerminateTLS": "box.example.ts.net"},
                "9443": {"HTTPS": true},
                "web": {"HTTPS": true},
                "7100": {}
            },
            "Web": {
                "box.example.ts.net:443": {"Handlers": {
                    "/notes": {"Text": "hello"},
                    "/": {"Proxy": "http://127.0.0.1:3000"}
                }},
                "box.example.ts.net:9443": {"Handlers": {"/m": {"Proxy": "http://127.0.0.1:47831/m"}}},
                "no-port": {"Handlers": {"/": {"Proxy": "http://127.0.0.1:3000"}}}
            },
            "AllowFunnel": {"box.example.ts.net:443": true},
            "Services": {"svc:web": {}},
            "Unknown": 1
        }"#;
        let status = parse_status(json).unwrap();
        assert_eq!(
            status.ports,
            [
                Port {
                    port: 80,
                    kind: Kind::Http
                },
                https(443),
                forward(5000, "127.0.0.1:47831", false),
                forward(6000, "localhost:7000", true),
                https(9443),
            ]
        );
        assert_eq!(
            status.mounts,
            [
                mount(443, "/", Some("http://127.0.0.1:3000")),
                mount(443, "/notes", None),
                mount(9443, "/m", Some("http://127.0.0.1:47831/m")),
            ]
        );
        assert!(status.services && status.funnel);
        let standing = status.standing(WINDOW);
        assert_eq!(standing.ours.as_deref(), Some(BOX));
        assert_eq!(
            standing.forwards,
            [
                "port 5000 forwards raw TCP to the window's port 47831: raw requests reach the \
              window with no login and no proxy header"
            ]
        );
        assert!(
            standing.held.is_empty() && standing.elsewhere.is_empty(),
            "{standing:?}"
        );
        // Funnel's flag counts only where it is on, and an empty service map is none.
        let off = parse_status(r#"{"AllowFunnel":{"box.example.ts.net:443":false},"Services":{}}"#)
            .unwrap();
        assert!(!off.funnel && !off.services);
    }

    #[test]
    fn merges_the_foreground_sessions() {
        let json = r#"{"TCP":{"443":{"HTTPS":true}},
            "Web":{"box.example.ts.net:443":{"Handlers":{"/":{"Proxy":"http://127.0.0.1:3000"}}}},
            "Foreground":{"a1b2c3":{
                "TCP":{"9443":{"HTTPS":true}},
                "Web":{"box.example.ts.net:9443":{"Handlers":{"/m":{"Proxy":"http://localhost:47831/m/"}}}},
                "AllowFunnel":{"box.example.ts.net:443":true}}}}"#;
        let status = parse_status(json).unwrap();
        assert_eq!(status.ports, [https(443), https(9443)]);
        assert_eq!(
            status.mounts,
            [
                mount(443, "/", Some("http://127.0.0.1:3000")),
                mount(9443, "/m", Some("http://localhost:47831/m/")),
            ]
        );
        assert!(status.funnel && !status.services);
        assert_eq!(status.standing(WINDOW).ours.as_deref(), Some(BOX));
    }

    #[test]
    fn ours_is_the_mount_enable_makes() {
        for proxy in [
            "http://127.0.0.1:47831/m",
            "http://localhost:47831/m/",
            "127.0.0.1:47831/m",
            "HTTP://LocalHost:47831/m",
        ] {
            let status = serving(vec![https(PORT)], vec![mount(PORT, "/m", Some(proxy))]);
            let standing = status.standing(WINDOW);
            assert_eq!(standing.ours.as_deref(), Some(BOX), "{proxy}");
            assert!(sentences(&standing).is_empty(), "{standing:?}");
        }
        // A `/m/` key is no mount Disable's `off` reaches: beside the window's port, not ours
        // (EP-31a's review).
        let slash = serving(
            vec![https(PORT)],
            vec![mount(PORT, "/m/", Some("http://localhost:47831/m/"))],
        );
        let standing = slash.standing(WINDOW);
        assert_eq!(standing.ours, None);
        assert_eq!(standing.elsewhere.len(), 1, "{standing:?}");
        // Without Serve's HTTPS on 9443, or for another window's port: not ours.
        let bare = serving(
            Vec::new(),
            vec![mount(PORT, "/m", Some("http://127.0.0.1:47831/m"))],
        );
        assert_eq!(bare.standing(WINDOW), Standing::default());
        assert_eq!(ours().standing(47832).ours, None);
        // Over https the window does not answer: not ours, and said.
        let secure = serving(
            vec![https(PORT)],
            vec![mount(PORT, "/m", Some("https://127.0.0.1:47831/m"))],
        );
        let standing = secure.standing(WINDOW);
        assert_eq!(standing.ours, None);
        assert_eq!(standing.elsewhere.len(), 1, "{standing:?}");
    }

    #[test]
    fn port_9443_held_otherwise_is_said_without_the_name() {
        let held = |status: Status| {
            let standing = status.standing(WINDOW);
            assert_eq!(standing.ours, None, "{standing:?}");
            assert!(
                standing.forwards.is_empty() && standing.elsewhere.is_empty(),
                "{standing:?}"
            );
            for sentence in sentences(&standing) {
                assert!(
                    !sentence.contains("box.example") && !sentence.contains("nas.example"),
                    "{sentence}"
                );
            }
            standing.held
        };
        let plain = Port {
            port: PORT,
            kind: Kind::Http,
        };
        assert_eq!(
            held(serving(
                vec![forward(PORT, "localhost:7000", false)],
                Vec::new()
            )),
            ["port 9443 forwards raw TCP to port 7000 of this machine"]
        );
        assert_eq!(
            held(serving(
                vec![forward(PORT, "nas.example.org:22", true)],
                Vec::new()
            )),
            ["port 9443 terminates TLS and forwards TCP to port 22 of another machine"]
        );
        assert_eq!(
            held(serving(vec![plain], Vec::new())),
            ["port 9443 serves plain HTTP"]
        );
        assert_eq!(
            held(serving(
                vec![https(PORT)],
                vec![mount(PORT, "/", Some("http://127.0.0.1:3000"))]
            )),
            [
                "port 9443 serves another path, /, for another service: it proxies to port 3000 of \
              this machine"
            ]
        );
        assert_eq!(
            held(serving(
                vec![https(PORT)],
                vec![mount(PORT, "/m", Some("http://127.0.0.1:3000/m"))]
            )),
            [
                "port 9443's path /m proxies to port 3000 of this machine, not the window's phone \
              pages"
            ]
        );
        assert_eq!(
            held(serving(vec![https(PORT)], vec![mount(PORT, "/m/", None)])),
            ["port 9443's path /m/ serves files or text, not the window's phone pages"]
        );
        // Beside the phone's mount, another path on 9443 still holds the port.
        let mut beside = ours();
        beside.mounts.push(mount(PORT, "/files", None));
        let standing = beside.standing(WINDOW);
        assert_eq!(standing.ours.as_deref(), Some(BOX));
        assert_eq!(
            standing.held,
            ["port 9443 serves files or text at another path, /files"]
        );
        // Another port's mounts, and a forward to another port, hold nothing.
        let other = serving(
            vec![https(443), forward(5000, "127.0.0.1:3000", false)],
            vec![mount(443, "/m", Some("http://127.0.0.1:3000/m"))],
        );
        assert_eq!(other.standing(WINDOW), Standing::default());
    }

    #[test]
    fn the_windows_port_exposed_elsewhere_is_counted_on_any_port() {
        let status = serving(
            vec![
                https(443),
                https(PORT),
                forward(5000, "127.0.0.1:47831", false),
                forward(6000, "localhost:47831", true),
                forward(7000, "[::1]:47831", false),
                forward(7100, "127.0.0.1:3000", false),
            ],
            vec![
                mount(443, "/", Some("http://127.0.0.1:47831")),
                mount(443, "/m", Some("http://localhost:47831/m")),
                mount(443, "/x", Some("http://127.0.0.1:3000")),
                mount(PORT, "/m", Some("http://127.0.0.1:47831/m")),
            ],
        );
        let standing = status.standing(WINDOW);
        assert_eq!(standing.ours.as_deref(), Some(BOX));
        let raw = "raw requests reach the window with no login and no proxy header";
        assert_eq!(
            standing.forwards,
            [
                format!("port 5000 forwards raw TCP to the window's port 47831: {raw}"),
                format!(
                    "port 6000 terminates TLS and forwards TCP to the window's port 47831: {raw}"
                ),
                format!("port 7000 forwards raw TCP to the window's port 47831: {raw}"),
            ]
        );
        assert_eq!(
            standing.elsewhere,
            [
                "port 443's path / proxies to the window's port 47831, which puts the window's \
                 other routes on the tailnet",
                "port 443's path /m also proxies to the window's phone pages, a mount Disable \
                 leaves alone",
            ]
        );
        assert!(standing.held.is_empty(), "{standing:?}");
        for sentence in sentences(&standing) {
            assert!(!sentence.contains("box.example"), "{sentence}");
        }
        // On 9443 itself, a mount to the window's root and a forward to its port are the window
        // exposed, not the port held.
        let root = serving(
            vec![https(PORT)],
            vec![mount(PORT, "/", Some("http://127.0.0.1:47831/"))],
        )
        .standing(WINDOW);
        assert_eq!(
            (root.ours, root.held.len(), root.elsewhere.len()),
            (None, 0, 1)
        );
        let raw =
            serving(vec![forward(PORT, "127.0.0.1:47831", false)], Vec::new()).standing(WINDOW);
        assert_eq!((raw.forwards.len(), raw.held.len()), (1, 0));
    }

    #[test]
    fn reads_the_node_narrowly() {
        let node = parse_node(&running()).unwrap();
        assert_eq!(
            node,
            Node {
                backend: "Running".to_owned(),
                login: Some(OWNER.to_owned()),
                name: Some(BOX.to_owned()),
                tagged: false,
                certificates: Some(true),
            }
        );
        let kept = format!("{node:?}");
        assert!(
            !kept.contains("phone") && !kept.contains("friend"),
            "nothing of the peers: {kept}"
        );
        // A tagged device, whose user is no person; an empty tag list is no tag.
        let tagged = r#"{"UserID":4242,"DNSName":"box.example.ts.net.","Tags":["tag:server"]}"#;
        assert!(
            parse_node(&node_json("Running", tagged, CERTS))
                .unwrap()
                .tagged
        );
        let untagged = r#"{"UserID":4242,"DNSName":"box.example.ts.net.","Tags":[]}"#;
        assert!(
            !parse_node(&node_json("Running", untagged, CERTS))
                .unwrap()
                .tagged
        );
        // Certificate domains: null or empty says none, absent says nothing.
        let certificates = |certs: Option<&str>| {
            parse_node(&node_json("Running", ME, certs))
                .unwrap()
                .certificates
        };
        assert_eq!(certificates(Some("null")), Some(false));
        assert_eq!(certificates(Some("[]")), Some(false));
        assert_eq!(certificates(None), None);
        // A stopped backend still reads; a user not in the map, or no name, is none.
        assert_eq!(
            parse_node(&node_json("Stopped", r#"{"UserID":5,"DNSName":"."}"#, None)).unwrap(),
            Node {
                backend: "Stopped".to_owned(),
                login: None,
                name: None,
                tagged: false,
                certificates: None,
            }
        );
        // The user found by its ID when the map's key is another.
        let by_id = r#"{"BackendState":"Running","Self":{"UserID":4242},
            "User":{"other":{"ID":4242,"LoginName":"owner@example.com"}}}"#;
        assert_eq!(parse_node(by_id).unwrap().login.as_deref(), Some(OWNER));
        // No backend state, or no JSON.
        assert!(parse_node("{}").unwrap_err().contains("BackendState"));
        assert!(
            parse_node("not json")
                .unwrap_err()
                .starts_with("tailscale printed no status")
        );
    }

    #[test]
    fn enable_plans_the_mount_after_two_reads() {
        let runner = Fake::new(&running(), "{}");
        let planned = plan(&facts(Verb::Enable), &runner);
        assert!(!planned.is_refused(), "{:?}", planned.problems);
        assert_eq!(
            runner.asked(),
            ["tailscale status --json", "tailscale serve status --json"]
        );
        let reads: Vec<Duration> = runner
            .asked
            .lock()
            .unwrap()
            .iter()
            .map(|command| command.timeout)
            .collect();
        assert_eq!(reads, [process::TIMEOUT, process::TIMEOUT]);
        let command = planned.command.as_ref().expect("a command");
        assert_eq!(
            command.line(),
            "tailscale serve --bg --https=9443 --set-path=/m http://127.0.0.1:47831/m"
        );
        assert_eq!(command.timeout, TIMEOUT);
        assert!(!command.long && command.folder.is_none());
        assert_eq!(planned.login.as_deref(), Some(OWNER));
        assert_eq!(planned.machine.as_deref(), Some(BOX));
        assert_eq!(
            planned.url.as_deref(),
            Some("https://box.example.ts.net:9443/m/")
        );
        assert_eq!(planned.words.len(), 4, "{:?}", planned.words);
        assert!(
            planned.words[0].contains("proxied to http://127.0.0.1:47831/m"),
            "{:?}",
            planned.words
        );
        assert!(planned.words[1].contains(OWNER), "{:?}", planned.words);
        assert!(planned.words[2].contains("never resets Serve's configuration"));
        assert_eq!(
            planned.warnings,
            ["A phone whose other VPN is on may not reach the tailnet, and so not the page."]
        );
        for word in planned.words.iter().chain(&planned.warnings) {
            assert!(!word.contains("box.example"), "{word}");
        }
    }

    #[test]
    fn enable_refuses_what_would_not_work() {
        let enable = facts(Verb::Enable);
        let problems = |facts: &Facts, runner: &Fake| {
            let planned = plan(facts, runner);
            assert!(planned.is_refused(), "{planned:?}");
            assert!(
                planned.command.is_none() && planned.words.is_empty(),
                "a refused plan runs nothing and says only why: {planned:?}"
            );
            for problem in &planned.problems {
                assert!(
                    !problem.contains("box.example") && !problem.contains("other.example"),
                    "{problem}"
                );
            }
            planned.problems
        };
        let one = |found: Vec<String>, words: &str| {
            assert_eq!(found.len(), 1, "{found:?}");
            assert!(found[0].contains(words), "{found:?}");
        };
        // The window's own refusal first, then a server that does not listen.
        let deaf = Facts {
            listening: false,
            refusals: vec!["a screenshot run acts on nothing".to_owned()],
            ..enable.clone()
        };
        let found = problems(&deaf, &Fake::new(&running(), "{}"));
        assert_eq!(found[0], "a screenshot run acts on nothing");
        assert!(
            found[1].contains("does not listen on port 47831"),
            "{found:?}"
        );
        // Tailscale stopped, a tagged device, no login, no name, a name outside ts.net, no
        // certificate domain.
        let node = |backend: &str, me: &str, certs: Option<&str>| {
            Fake::new(&node_json(backend, me, certs), "{}")
        };
        one(
            problems(&enable, &node("Stopped", ME, CERTS)),
            "Tailscale is Stopped: the tailnet is not up",
        );
        let tagged = r#"{"UserID":4242,"DNSName":"box.example.ts.net.","Tags":["tag:server"]}"#;
        one(
            problems(&enable, &node("Running", tagged, CERTS)),
            "set it by hand with crw setting owner_login",
        );
        let nobody = r#"{"UserID":5,"DNSName":"box.example.ts.net."}"#;
        one(
            problems(&enable, &node("Running", nobody, CERTS)),
            "names no login for this machine's user",
        );
        let nameless = r#"{"UserID":4242,"DNSName":""}"#;
        one(
            problems(&enable, &node("Running", nameless, CERTS)),
            "no DNS name for this machine: MagicDNS may be off",
        );
        let elsewhere = r#"{"UserID":4242,"DNSName":"box.example.org."}"#;
        one(
            problems(&enable, &node("Running", elsewhere, CERTS)),
            "no tailnet name of the form",
        );
        one(
            problems(&enable, &node("Running", ME, Some("null"))),
            "turn HTTPS certificates on in the tailnet's DNS settings",
        );
        // Tailscale's status unread: Serve is not asked.
        let mut silent = Fake::new("", "{}");
        silent.node = Err("no answer within 20s".to_owned());
        assert_eq!(
            problems(&enable, &silent),
            ["Tailscale's status did not read: no answer within 20s"]
        );
        assert_eq!(silent.asked(), ["tailscale status --json"]);
        let mut down = Fake::new("", "{}");
        down.node = Ok(failed(1, "", "failed to connect to the local daemon"));
        one(
            problems(&enable, &down),
            "tailscale status failed (exit code 1): failed to connect to the local daemon",
        );
        // A status that exits with an error and still prints its JSON reads.
        let mut stopped = Fake::new("", "{}");
        stopped.node = Ok(failed(1, &node_json("Stopped", ME, CERTS), ""));
        one(problems(&enable, &stopped), "Tailscale is Stopped");
        // Serve's status failing, not reading, not answering.
        let mut denied = Fake::new(&running(), "");
        denied.serve = Ok(failed(1, "", "access denied"));
        one(
            problems(&enable, &denied),
            "tailscale serve status failed (exit code 1): access denied",
        );
        one(
            problems(&enable, &Fake::new(&running(), "garbage")),
            "Serve's configuration did not read: tailscale serve printed no configuration",
        );
        let mut late = Fake::new(&running(), "");
        late.serve = Err("no answer within 20s".to_owned());
        one(
            problems(&enable, &late),
            "Serve's configuration did not read: no answer within 20s",
        );
        // Port 9443 held, the window's port forwarded or mounted elsewhere.
        one(
            problems(
                &enable,
                &Fake::new(&running(), r#"{"TCP":{"9443":{"HTTP":true}}}"#),
            ),
            "port 9443 serves plain HTTP: the phone's path needs port 9443 to itself",
        );
        let by_hand = "; take it off by hand first: the tool never resets Serve's configuration";
        let forwarded = problems(
            &enable,
            &Fake::new(
                &running(),
                r#"{"TCP":{"5000":{"TCPForward":"127.0.0.1:47831"}}}"#,
            ),
        );
        assert!(
            forwarded.len() == 1
                && forwarded[0].starts_with("port 5000 forwards raw TCP")
                && forwarded[0].ends_with(by_hand),
            "{forwarded:?}"
        );
        let mounted = problems(
            &enable,
            &Fake::new(
                &running(),
                r#"{"TCP":{"443":{"HTTPS":true}},"Web":{"box.example.ts.net:443":
                    {"Handlers":{"/":{"Proxy":"http://127.0.0.1:47831"}}}}}"#,
            ),
        );
        assert!(
            mounted.len() == 1
                && mounted[0].starts_with("port 443's path / proxies to the window's port")
                && mounted[0].ends_with(by_hand),
            "{mounted:?}"
        );
        // The phone's mount under another machine's name.
        one(
            problems(
                &enable,
                &Fake::new(&running(), &OURS.replace(BOX, "other.example.ts.net")),
            ),
            "another machine name than this machine's",
        );
    }

    #[test]
    fn enable_with_the_mount_there_writes_the_registry_only() {
        let runner = Fake::new(&running(), OURS);
        let planned = plan(&facts(Verb::Enable), &runner);
        assert!(!planned.is_refused(), "{:?}", planned.problems);
        assert!(planned.command.is_none());
        assert!(
            planned.words[0].starts_with("Serve already mounts the phone's path"),
            "{:?}",
            planned.words
        );
        assert_eq!(planned.login.as_deref(), Some(OWNER));
        assert_eq!(planned.machine.as_deref(), Some(BOX));
        assert_eq!(runner.asked().len(), 2, "the reads alone");
    }

    #[test]
    fn enable_warns_of_another_login_and_of_funnel() {
        let funnel = r#"{"TCP":{"443":{"HTTPS":true}},
            "Web":{"box.example.ts.net:443":{"Handlers":{"/":{"Proxy":"http://127.0.0.1:3000"}}}},
            "AllowFunnel":{"box.example.ts.net:443":true}}"#;
        let other = Facts {
            login: Some("someone@example.org".to_owned()),
            ..facts(Verb::Enable)
        };
        let planned = plan(&other, &Fake::new(&running(), funnel));
        assert!(!planned.is_refused(), "{:?}", planned.problems);
        assert_eq!(planned.warnings.len(), 3, "{:?}", planned.warnings);
        assert_eq!(
            planned.warnings[0],
            "The registry names another login: Enable replaces it with owner@example.com."
        );
        assert!(planned.warnings[1].starts_with("Funnel is on for another entry"));
        // The same login in another case is no other login.
        let same = Facts {
            login: Some(" Owner@Example.com ".to_owned()),
            ..facts(Verb::Enable)
        };
        assert_eq!(plan(&same, &Fake::new(&running(), "{}")).warnings.len(), 1);
    }

    #[test]
    fn disable_takes_the_mount_off_or_the_record_out() {
        let runner = Fake::new("never asked", OURS);
        let planned = plan(&recorded(), &runner);
        assert!(!planned.is_refused(), "{:?}", planned.problems);
        assert_eq!(
            runner.asked(),
            ["tailscale serve status --json"],
            "Serve's status alone"
        );
        let command = planned.command.as_ref().expect("a command");
        assert_eq!(
            command.line(),
            "tailscale serve --https=9443 --set-path=/m off"
        );
        assert_eq!(command.timeout, TIMEOUT);
        assert_eq!(planned.login, None);
        assert_eq!(planned.machine.as_deref(), Some(BOX));
        assert_eq!(
            planned.url.as_deref(),
            Some("https://box.example.ts.net:9443/m/")
        );
        assert_eq!(
            planned.words.last().map(String::as_str),
            Some("The login stays in the registry for the next Enable.")
        );
        // The mount with no record: the command, the URL from the mount.
        let bare = plan(&facts(Verb::Disable), &Fake::new("", OURS));
        assert!(bare.command.is_some() && !bare.is_refused(), "{bare:?}");
        assert_eq!(bare.machine, None);
        assert_eq!(
            bare.url.as_deref(),
            Some("https://box.example.ts.net:9443/m/")
        );
        // The record with no mount: the registry alone.
        let record = plan(&recorded(), &Fake::new("", "{}"));
        assert!(
            !record.is_refused() && record.command.is_none(),
            "{record:?}"
        );
        assert!(record.words[0].starts_with("Serve holds no mount of the phone's path"));
        // Neither: nothing to take off.
        let off = plan(&facts(Verb::Disable), &Fake::new("", "{}"));
        assert!(off.is_refused());
        assert!(off.problems[0].starts_with("remote view is off"), "{off:?}");
        // Serve's status unread, and the window's refusal.
        let mut denied = Fake::new("", OURS);
        denied.serve = Ok(failed(1, "", "access denied"));
        let unread = plan(&recorded(), &denied);
        assert!(unread.is_refused() && unread.problems[0].contains("access denied"));
        let busy = Facts {
            refusals: vec!["a screenshot run acts on nothing".to_owned()],
            ..recorded()
        };
        assert!(plan(&busy, &Fake::new("", OURS)).is_refused());
        for word in planned
            .words
            .iter()
            .chain(&record.words)
            .chain(&off.problems)
        {
            assert!(!word.contains("box.example"), "{word}");
        }
    }

    #[test]
    fn a_run_writes_the_registry_once_it_went_through_and_keeps_a_row() {
        let scratch = Scratch::new("serve-run");
        let store = store(&scratch);
        let ledger = scratch.0.join("local").join("ledger.sqlite");
        // Enable from off: Serve's change, Serve read again, then the login and the name in one
        // change.
        let runner = Fake::then(&running(), "{}", OURS);
        let enable = plan(&facts(Verb::Enable), &runner);
        let ran = run(&enable, &runner, Some(&store), Some(&ledger), TODAY).expect("it ran");
        assert_eq!(ran.end, End::Done);
        assert_eq!(ran.unwritten, None);
        assert_eq!(ran.unlogged, None);
        assert!(
            ran.printed.starts_with("Available within your tailnet:")
                && ran.printed.ends_with("in the background."),
            "{}",
            ran.printed
        );
        assert_eq!(
            ran.command.as_deref(),
            Some("tailscale serve --bg --https=9443 --set-path=/m http://127.0.0.1:47831/m")
        );
        assert_eq!(
            runner.asked(),
            [
                "tailscale status --json",
                "tailscale serve status --json",
                "tailscale serve --bg --https=9443 --set-path=/m http://127.0.0.1:47831/m",
                "tailscale serve status --json",
            ],
            "the plan's two reads, the change, and Serve read again"
        );
        assert_eq!(
            ran.url.as_deref(),
            Some("https://box.example.ts.net:9443/m/")
        );
        assert_eq!(
            ran.words(),
            "Remote view enabled: Serve mounts the phone's path on port 9443"
        );
        let settings = store.load().unwrap().registry.settings;
        assert_eq!(settings.owner_login.as_deref(), Some(OWNER));
        assert_eq!(settings.remote_view(), Some(BOX));
        // Disable with the mount there: the name out, the login kept.
        let runner = Fake::then("", OURS, "{}");
        let disable = plan(&recorded(), &runner);
        let ran = run(&disable, &runner, Some(&store), Some(&ledger), TODAY).unwrap();
        assert_eq!(ran.end, End::Done);
        assert_eq!(
            ran.words(),
            "Remote view disabled: Serve's mount of the phone's path is off"
        );
        let settings = store.load().unwrap().registry.settings;
        assert_eq!(settings.remote_view(), None);
        assert_eq!(settings.owner_login.as_deref(), Some(OWNER));
        // A failed change writes nothing and is said in Serve's words, never retried.
        let before = fs::read(&store.path).unwrap();
        let mut failing = Fake::new(&running(), "{}");
        failing.change = Ok(failed(
            1,
            "",
            "error: a listener already exists for box.example.ts.net:9443\n",
        ));
        let refused_by_serve = plan(&facts(Verb::Enable), &failing);
        let ran = run(
            &refused_by_serve,
            &failing,
            Some(&store),
            Some(&ledger),
            TODAY,
        )
        .unwrap();
        assert_eq!(
            ran.end,
            End::Failed(
                "tailscale serve failed (exit code 1): error: a listener already exists for \
                 box.example.ts.net:9443"
                    .to_owned()
            )
        );
        assert_eq!(ran.unwritten, None);
        assert_eq!(
            ran.words(),
            "Remote view was not enabled: tailscale serve failed (exit code 1): error: a \
             listener already exists for <this machine's name>"
        );
        assert_eq!(failing.asked().len(), 3, "two reads and one change");
        assert_eq!(
            fs::read(&store.path).unwrap(),
            before,
            "the registry untouched"
        );
        // A refused plan runs nothing and keeps no row.
        let refused = Plan {
            problems: vec!["a screenshot run acts on nothing".to_owned()],
            ..enable.clone()
        };
        assert!(run(&refused, &failing, Some(&store), Some(&ledger), TODAY).is_none());
        assert_eq!(failing.asked().len(), 3);
        // The rows, the newest first: kind remote, no project, the verb, the outcome and the
        // command, never the name or Serve's printed words.
        let rows = Ledger::open_read_only(&ledger)
            .unwrap()
            .events(remote::EVENT_KIND, None)
            .unwrap();
        let details: Vec<&str> = rows
            .iter()
            .map(|row| row.detail.as_deref().unwrap_or_default())
            .collect();
        assert_eq!(details.len(), 3, "{details:?}");
        assert_eq!(
            details[2],
            "enable: Remote view enabled: Serve mounts the phone's path on port 9443 · tailscale \
             serve --bg --https=9443 --set-path=/m http://127.0.0.1:47831/m"
        );
        assert_eq!(
            details[1],
            "disable: Remote view disabled: Serve's mount of the phone's path is off · tailscale \
             serve --https=9443 --set-path=/m off"
        );
        assert!(
            details[0].starts_with("enable: Remote view was not enabled: tailscale serve failed"),
            "{}",
            details[0]
        );
        for row in &rows {
            assert_eq!(row.kind, remote::EVENT_KIND);
            assert_eq!(row.project, None);
        }
        for detail in &details {
            assert!(
                !detail.contains("box.example") && !detail.contains("Serve started"),
                "{detail}"
            );
        }
    }

    #[test]
    fn a_run_with_nothing_to_run_writes_the_registry_alone() {
        let scratch = Scratch::new("serve-run-registry");
        let store = store(&scratch);
        let ledger = scratch.0.join("ledger.sqlite");
        // Enable with the mount there: no command, the registry written.
        let runner = Fake::new(&running(), OURS);
        let enable = plan(&facts(Verb::Enable), &runner);
        let ran = run(&enable, &runner, Some(&store), Some(&ledger), TODAY).unwrap();
        assert_eq!(ran.command, None);
        assert_eq!(ran.end, End::Done);
        assert_eq!(runner.asked().len(), 2, "the reads alone");
        assert_eq!(
            ran.words(),
            "Remote view enabled: Serve already mounted the phone's path; the login and the \
             machine's name are in the registry"
        );
        assert_eq!(
            store.load().unwrap().registry.settings.remote_view(),
            Some(BOX)
        );
        // Disable with no mount: the record out, the login kept.
        let runner = Fake::new("", "{}");
        let disable = plan(&recorded(), &runner);
        let ran = run(&disable, &runner, Some(&store), Some(&ledger), TODAY).unwrap();
        assert_eq!(
            ran.words(),
            "Remote view disabled: the registry's record is out (Serve held no mount)"
        );
        let settings = store.load().unwrap().registry.settings;
        assert_eq!(settings.remote_view(), None);
        assert_eq!(settings.owner_login.as_deref(), Some(OWNER));
        let rows = Ledger::open_read_only(&ledger)
            .unwrap()
            .events(remote::EVENT_KIND, None)
            .unwrap();
        assert_eq!(rows.len(), 2);
        assert!(
            rows.iter().all(|row| row
                .detail
                .as_deref()
                .is_some_and(|detail| detail.ends_with(" · no command"))),
            "{rows:?}"
        );
        // A registry that does not take the change: said at the end of the words.
        let unwritable = Store::new(
            scratch.0.join("folder"),
            scratch.0.join("backups"),
            store.defaults.clone(),
        );
        fs::create_dir_all(&unwritable.path).unwrap();
        let ran = run(&enable, &Fake::new("", ""), Some(&unwritable), None, TODAY).unwrap();
        assert_eq!(ran.end, End::Done);
        let why = ran.unwritten.clone().expect("not written");
        assert!(why.starts_with("the registry was not written: "), "{why}");
        assert!(
            ran.words().starts_with(
                "Remote view was not enabled: Serve already mounted the phone's path; the \
                 registry was not written: "
            ),
            "{}",
            ran.words()
        );
        assert_eq!(ran.unlogged, None, "no ledger, no row");
    }

    #[test]
    fn the_doctors_line_says_each_state() {
        let off: Result<Status, String> = Ok(Status::default());
        let on: Result<Status, String> = Ok(ours());
        let unread: Result<Status, String> =
            Err("tailscale serve status failed (exit code 1): no daemon".to_owned());
        let cases = [
            (
                line(None, None, WINDOW),
                Level::Note,
                "off; Serve's status not read (Tailscale not found)",
            ),
            (
                line(None, Some(BOX), WINDOW),
                Level::Warn,
                "on in the registry, but Serve's status was not read: Tailscale not found",
            ),
            (
                line(Some(&unread), None, WINDOW),
                Level::Note,
                "Serve's status did not read: tailscale serve status failed (exit code 1): no \
                 daemon",
            ),
            (
                line(Some(&unread), Some(BOX), WINDOW),
                Level::Warn,
                "on in the registry, but Serve's status did not read: tailscale serve status \
                 failed (exit code 1): no daemon",
            ),
            (
                line(Some(&off), None, WINDOW),
                Level::Note,
                "off: Enable remote view (the Diagnostics pane or the tray's menu) mounts the \
                 phone's path on the tailnet",
            ),
            (
                line(Some(&on), Some(BOX), WINDOW),
                Level::Ok,
                "on at https://box.example.ts.net:9443/m/: Serve proxies the phone's path alone",
            ),
            (
                line(Some(&on), Some("other.example.ts.net"), WINDOW),
                Level::Warn,
                "Serve mounts the phone's path under another machine name than the registry's: \
                 Disable, then Enable again",
            ),
            (
                line(Some(&on), None, WINDOW),
                Level::Warn,
                "Serve mounts the phone's path, but the registry has no record of it: Enable \
                 remote view writes the login and the machine's name",
            ),
            (
                line(Some(&off), Some(BOX), WINDOW),
                Level::Warn,
                "on in the registry while Serve holds no mount of the phone's path: Enable \
                 again, or Disable to take the record out",
            ),
        ];
        for ((level, words), want_level, want_words) in &cases {
            assert_eq!((level, words.as_str()), (want_level, *want_words));
        }
        // Case and a final dot are no other name.
        assert_eq!(
            line(Some(&on), Some("BOX.example.ts.net."), WINDOW).0,
            Level::Ok
        );
        // A forward to the window's port fails the line, another mount to it warns, and the URL
        // goes once anything else is said.
        let mut exposed = ours();
        exposed.ports.push(forward(5000, "127.0.0.1:47831", false));
        exposed
            .mounts
            .push(mount(443, "/", Some("http://127.0.0.1:47831")));
        let exposed = line(Some(&Ok(exposed)), Some(BOX), WINDOW);
        assert_eq!(
            exposed,
            (
                Level::Fail,
                "on: Serve proxies the phone's path; port 5000 forwards raw TCP to the window's \
                 port 47831: raw requests reach the window with no login and no proxy header; \
                 take it off by hand; port 443's path / proxies to the window's port 47831, \
                 which puts the window's other routes on the tailnet; take it off by hand"
                    .to_owned()
            )
        );
        let mut beside = ours();
        beside
            .mounts
            .push(mount(443, "/", Some("http://127.0.0.1:47831")));
        assert_eq!(line(Some(&Ok(beside)), Some(BOX), WINDOW).0, Level::Warn);
        // What holds port 9443 is a note while the mount is not there.
        let plain = serving(
            vec![Port {
                port: PORT,
                kind: Kind::Http,
            }],
            Vec::new(),
        );
        let held = line(Some(&Ok(plain)), None, WINDOW);
        assert_eq!(held.0, Level::Note);
        assert!(
            held.1.ends_with(
                "; port 9443 serves plain HTTP: the phone's path needs port 9443 to itself"
            ),
            "{}",
            held.1
        );
        // Serve's words that name the machine are masked.
        let named: Result<Status, String> = Err(
            "tailscale serve status failed (exit code 1): no config for box.example.ts.net:9443"
                .to_owned(),
        );
        let masked = line(Some(&named), None, WINDOW);
        assert!(
            masked.1.ends_with("no config for <this machine's name>"),
            "{}",
            masked.1
        );
        // Only the Ok line names the machine.
        let all = cases
            .iter()
            .map(|(found, _, _)| found)
            .chain([&exposed, &held, &masked]);
        for (level, words) in all {
            assert!(
                *level == Level::Ok || !words.contains("box.example"),
                "{level:?} {words}"
            );
        }
    }

    /// A command that ends well with nothing changed (a prompt answered by the empty input) is
    /// no success: Serve read again decides, and nothing is written (EP-31a's review).
    #[test]
    fn a_change_that_changed_nothing_writes_nothing() {
        let scratch = Scratch::new("serve-run-unchanged");
        let store = store(&scratch);
        let ledger = scratch.0.join("ledger.sqlite");
        let still_off = Fake::new(&running(), "{}");
        let enable = plan(&facts(Verb::Enable), &still_off);
        let ran = run(&enable, &still_off, Some(&store), Some(&ledger), TODAY).unwrap();
        match &ran.end {
            End::Failed(why) => assert!(
                why.contains("read again, does not show the phone's mount"),
                "{why}"
            ),
            other => panic!("{other:?}"),
        }
        assert_eq!(ran.printed, STARTED.trim(), "Serve's words still shown");
        assert_eq!(store.load().unwrap().registry.settings.owner_login, None);
        let still_on = Fake::new("", OURS);
        let disable = plan(&recorded(), &still_on);
        let ran = run(&disable, &still_on, Some(&store), Some(&ledger), TODAY).unwrap();
        assert!(
            matches!(&ran.end, End::Failed(why) if why.contains("still shows the phone's mount")),
            "{:?}",
            ran.end
        );
    }

    /// Funnel on the phone's port puts the pages on the internet: Enable is refused and the
    /// doctor fails; Funnel on another port is a warning alone (EP-31a's review; D-28).
    #[test]
    fn funnel_on_the_phones_port_refuses_enable_and_fails_the_doctor() {
        let funnelled = OURS.replacen(
            "\"Web\"",
            "\"AllowFunnel\":{\"box.example.ts.net:9443\":true},\"Web\"",
            1,
        );
        let status = parse_status(&funnelled).unwrap();
        assert_eq!(status.funnel_ports, [PORT]);
        let standing = status.standing(WINDOW);
        assert!(standing.funnel);
        assert!(standing.ours.is_some(), "the mount itself is the tool's");
        let planned = plan(&facts(Verb::Enable), &Fake::new(&running(), &funnelled));
        assert!(
            planned
                .problems
                .iter()
                .any(|why| why.starts_with("Funnel is on for port 9443")),
            "{:?}",
            planned.problems
        );
        assert!(
            !planned
                .warnings
                .iter()
                .any(|words| words.contains("another entry")),
            "{:?}",
            planned.warnings
        );
        let (level, words) = line(Some(&Ok(status)), Some(BOX), WINDOW);
        assert_eq!(level, Level::Fail);
        assert!(words.contains("Funnel is on for port 9443"), "{words}");
        assert!(!words.contains("box.example"), "{words}");
        // Funnel for the other service on 443 alone: a warning, Enable goes on.
        let elsewhere = "{\"TCP\":{\"443\":{\"HTTPS\":true}},\"AllowFunnel\":\
                         {\"box.example.ts.net:443\":true}}";
        let planned = plan(&facts(Verb::Enable), &Fake::new(&running(), elsewhere));
        assert!(!planned.is_refused(), "{:?}", planned.problems);
        assert!(
            planned
                .warnings
                .iter()
                .any(|words| words.contains("another entry")),
            "{:?}",
            planned.warnings
        );
    }

    /// The phone's mount an earlier window made, before the window's port changed, is the
    /// tool's own: Disable takes it off, Enable is refused until then, and the doctor says so
    /// (EP-31a's review).
    #[test]
    fn an_earlier_windows_mount_is_taken_off_by_disable() {
        let earlier = OURS.replace("127.0.0.1:47831/m", "127.0.0.1:47900/m");
        let standing = parse_status(&earlier).unwrap().standing(WINDOW);
        assert_eq!(standing.ours, None);
        assert_eq!(standing.earlier, Some((BOX.to_owned(), 47900)));
        // It holds the port whoever made it.
        assert_eq!(standing.held.len(), 1, "{:?}", standing.held);
        assert_eq!(standing.earlier_of(Some(BOX)), Some(47900));
        assert_eq!(standing.earlier_of(None), None);
        assert_eq!(standing.earlier_of(Some("other.example.ts.net")), None);
        // With the registry's record naming its host it is the tool's own: Enable says Disable
        // takes it off, and Disable does.
        let with_record = Facts {
            machine: Some(BOX.to_owned()),
            ..facts(Verb::Enable)
        };
        let enable = plan(&with_record, &Fake::new(&running(), &earlier));
        assert!(
            enable
                .problems
                .iter()
                .any(|why| why.contains("the one an earlier window made")),
            "{:?}",
            enable.problems
        );
        let disable = plan(&recorded(), &Fake::new("", &earlier));
        assert_eq!(
            disable.command.as_ref().map(Command::line).as_deref(),
            Some("tailscale serve --https=9443 --set-path=/m off")
        );
        let (level, words) = line(Some(&parse_status(&earlier)), Some(BOX), WINDOW);
        assert_eq!(level, Level::Warn);
        assert!(words.contains("an earlier window's"), "{words}");
        // Without the record it may be another service's: Enable is refused by the held port
        // alone, Disable leaves it alone, and the doctor notes the port held.
        let enable = plan(&facts(Verb::Enable), &Fake::new(&running(), &earlier));
        assert!(enable.is_refused());
        assert!(
            !enable
                .problems
                .iter()
                .any(|why| why.contains("earlier window")),
            "{:?}",
            enable.problems
        );
        let disable = plan(&facts(Verb::Disable), &Fake::new("", &earlier));
        assert!(disable.is_refused() && disable.command.is_none());
        let (level, words) = line(Some(&parse_status(&earlier)), None, WINDOW);
        assert_eq!(level, Level::Note);
        assert!(!words.contains("earlier window"), "{words}");
    }

    /// Only the mount key Enable writes, `/m`, is the tool's: a `/m/` key is one Disable's `off`
    /// does not reach, so it counts beside the window's port, never as the mount (EP-31a's
    /// review).
    #[test]
    fn a_slash_m_mount_is_not_the_tools() {
        let slash = OURS.replace("\"/m\":", "\"/m/\":");
        let standing = parse_status(&slash).unwrap().standing(WINDOW);
        assert_eq!(standing.ours, None);
        assert_eq!(standing.elsewhere.len(), 1, "{:?}", standing.elsewhere);
        assert!(
            plan(&facts(Verb::Enable), &Fake::new(&running(), &slash)).is_refused(),
            "Enable is refused while it stands"
        );
    }
}
