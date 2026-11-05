//! The usage meter (D-23; EP-17): how much of its limits the account has used, from two sources.
//!
//! - **The endpoint** `GET /api/oauth/usage`, undocumented and treated as breakable, gives the
//!   weekly limit, the scoped weekly limits (Fable's among them), the five-hour window while one
//!   is active, the week's use by surface and extra usage ([`Reading`]). The window polls it on a
//!   worker thread (`app::meter`) every [`EVERY`], soon after a session the tool launched ends,
//!   and with a backoff while it fails ([`Schedule`]).
//! - **The status lines** of the sessions that run one (EP-12) give the weekly and five-hour
//!   figures, a second series ([`Line`]).
//!
//! The token the poll sends is read from Claude Code's credentials file at each poll
//! ([`read_token`]) into a [`Secret`], which never prints and is overwritten when it drops; it is
//! never written, logged or shown. While the endpoint answers, its figures stand; when it fails,
//! the meter shows the newest of its last reading and the status lines', marked degraded with the
//! reason ([`Gauge::figures`]). Each reading is a `usage_sample` row of the ledger, the endpoint's
//! last one also its cache row ([`CACHE_SOURCE`]), and the [`WARN`] and [`ALARM`] marks of the
//! weekly and Fable limits are notified once per limit window ([`Alerts`]).

use std::collections::BTreeMap;
use std::fmt;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::atomic::{Ordering, compiler_fence};

use chrono::{DateTime, TimeDelta, Utc};
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;

use crate::badges::rfc3339;
use crate::ledger::Sample;
use crate::signals::{self, StatusLine};

pub mod http;

/// The endpoint (D-23).
pub const ENDPOINT: &str = "https://api.anthropic.com/api/oauth/usage";

/// The beta header the endpoint asks for.
pub const BETA: &str = "oauth-2025-04-20";

/// The pace of the polls while the endpoint answers.
pub const EVERY: TimeDelta = TimeDelta::minutes(5);

/// The first poll after the window starts.
pub const FIRST: TimeDelta = TimeDelta::seconds(5);

/// A session's end brings the next poll forward, to no sooner than this after the last one.
pub const AFTER_END: TimeDelta = TimeDelta::minutes(1);

/// After a failure the endpoint is asked again after 1, 2, 4, 8 and 16 minutes, then every 30.
const BACKOFF_MINUTES: [i64; 5] = [1, 2, 4, 8, 16];
const BACKOFF_CAP: TimeDelta = TimeDelta::minutes(30);

/// The longest wait a `Retry-After` is granted.
const LONGEST_WAIT: TimeDelta = TimeDelta::days(1);

/// A problem on this machine (no credentials, an expired token) is looked at again this often:
/// no request was sent, so there is nothing to back off from.
const LOCAL_RETRY: TimeDelta = TimeDelta::minutes(1);

/// The endpoint's figures stand while its last reading is younger than this (three polls).
pub const FRESH_FOR: TimeDelta = TimeDelta::minutes(15);

/// A token this close to its expiry is taken as expired.
const EXPIRY_MARGIN: TimeDelta = TimeDelta::seconds(60);

/// The ledger's cache rows: the endpoint's last reading, and the notifications sent per window.
pub const CACHE_SOURCE: &str = "usage";
pub const ALERTS_SOURCE: &str = "usage-alerts";

/// The ledger's `event` kind of the meter's changes of state.
pub const EVENT_KIND: &str = "usage";

/// The marks notified (D-21, D-23), in percent.
pub const WARN: f64 = 80.0;
pub const ALARM: f64 = 95.0;

/// The most of a schema change's words: the body's shape, never its values.
const SHAPE_CAP: usize = 600;

/// The most of an error answer's message kept in its words.
const MESSAGE_CAP: usize = 160;

/// A secret string: the token, or the header block that carries it. It never prints (`Debug`
/// shows `[redacted]`, and it has no `Display`), and its bytes are overwritten when it drops.
pub struct Secret(String);

impl Secret {
    /// The text, for the request only.
    pub fn expose(&self) -> &str {
        &self.0
    }
}

impl fmt::Debug for Secret {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Secret([redacted])")
    }
}

impl Drop for Secret {
    fn drop(&mut self) {
        // SAFETY: zeros keep the text valid UTF-8.
        wipe(unsafe { self.0.as_bytes_mut() });
    }
}

impl<'de> Deserialize<'de> for Secret {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        String::deserialize(deserializer).map(Secret)
    }
}

/// Overwrites `bytes` with zeros, in writes the compiler keeps.
pub fn wipe(bytes: &mut [u8]) {
    for byte in bytes.iter_mut() {
        // SAFETY: a valid, aligned reference.
        unsafe { std::ptr::write_volatile(byte, 0) };
    }
    compiler_fence(Ordering::SeqCst);
}

/// The same for a wide string.
pub fn wipe_wide(units: &mut [u16]) {
    for unit in units.iter_mut() {
        // SAFETY: a valid, aligned reference.
        unsafe { std::ptr::write_volatile(unit, 0) };
    }
    compiler_fence(Ordering::SeqCst);
}

/// Claude Code's folder: `CLAUDE_CONFIG_DIR`, else `.claude` in the profile.
pub fn claude_dir() -> Option<PathBuf> {
    let var = |name| std::env::var_os(name).filter(|value| !value.is_empty());
    var("CLAUDE_CONFIG_DIR")
        .map(PathBuf::from)
        .or_else(|| var("USERPROFILE").map(|home| PathBuf::from(home).join(".claude")))
}

/// Claude Code's credentials file, which only the poll reads, at poll time (D-23).
pub fn credentials_file() -> Option<PathBuf> {
    claude_dir().map(|dir| dir.join(".credentials.json"))
}

#[derive(Deserialize)]
struct Credentials {
    #[serde(rename = "claudeAiOauth")]
    oauth: Option<OAuth>,
}

#[derive(Deserialize)]
struct OAuth {
    #[serde(rename = "accessToken")]
    access_token: Option<Secret>,
    /// Milliseconds since the epoch.
    #[serde(rename = "expiresAt")]
    expires_at: Option<f64>,
}

/// Why a poll gave no reading.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Problem {
    /// The registry's switch is off (`settings.usage_endpoint`).
    Off,
    NoCredentials,
    /// The credentials file did not read, in words that hold none of its text.
    Credentials(String),
    /// No claude.ai sign-in in the file (an API key, or signed out).
    NoToken,
    Expired(DateTime<Utc>),
    Network(String),
    /// An answer other than 2xx: its status, the wait it asked for in seconds, and its error's
    /// type and message when it gave them.
    Status {
        code: u16,
        retry_after_s: Option<u64>,
        message: Option<String>,
    },
    /// An answer the meter cannot read, with the body's shape.
    Schema(String),
}

impl Problem {
    pub fn words(&self) -> String {
        match self {
            Problem::Off => {
                "the endpoint is switched off (crw setting usage_endpoint on)".to_owned()
            }
            Problem::NoCredentials => {
                "no Claude Code credentials file: sign in to Claude Code".to_owned()
            }
            Problem::Credentials(why) => format!("the credentials file did not read: {why}"),
            Problem::NoToken => "the credentials file holds no claude.ai sign-in".to_owned(),
            Problem::Expired(at) => format!(
                "the token expired at {} UTC; Claude Code renews it when it next runs",
                at.format("%Y-%m-%d %H:%M")
            ),
            Problem::Network(why) => format!("the endpoint did not answer: {why}"),
            Problem::Status { code, message, .. } => {
                let said = message
                    .as_deref()
                    .map(|message| format!(": {message}"))
                    .unwrap_or_default();
                match code {
                    401 | 403 => format!("the endpoint refused the token (HTTP {code}){said}"),
                    429 => format!("the endpoint asks the meter to wait (HTTP 429){said}"),
                    _ => format!("the endpoint answered HTTP {code}{said}"),
                }
            }
            Problem::Schema(why) => format!("the endpoint's answer did not read: {why}"),
        }
    }

    /// The wait the endpoint asked for.
    pub fn retry_after(&self) -> Option<TimeDelta> {
        match self {
            Problem::Status {
                retry_after_s: Some(seconds),
                ..
            } => Some(TimeDelta::seconds(
                i64::try_from(*seconds).map_or(LONGEST_WAIT.num_seconds(), |s| {
                    s.min(LONGEST_WAIT.num_seconds())
                }),
            )),
            _ => None,
        }
    }

    /// A problem on this machine: no request was sent.
    pub fn is_local(&self) -> bool {
        matches!(
            self,
            Problem::Off
                | Problem::NoCredentials
                | Problem::Credentials(_)
                | Problem::NoToken
                | Problem::Expired(_)
        )
    }
}

/// The access token in `path`, read now; an expired one is not handed out.
pub fn read_token(path: &Path, now: DateTime<Utc>) -> Result<Secret, Problem> {
    let mut bytes = match std::fs::read(path) {
        Ok(bytes) => bytes,
        Err(e) if e.kind() == io::ErrorKind::NotFound => return Err(Problem::NoCredentials),
        Err(e) => return Err(Problem::Credentials(e.kind().to_string())),
    };
    let parsed = serde_json::from_slice::<Credentials>(&bytes);
    wipe(&mut bytes);
    // The error's category and place only: a type error's words could quote the file.
    let credentials = parsed.map_err(|e| {
        Problem::Credentials(format!(
            "not the JSON expected ({:?} at line {}, column {})",
            e.classify(),
            e.line(),
            e.column()
        ))
    })?;
    let oauth = credentials.oauth.ok_or(Problem::NoToken)?;
    let token = oauth
        .access_token
        .filter(|token| !token.expose().trim().is_empty())
        .ok_or(Problem::NoToken)?;
    if let Some(at) = oauth.expires_at.and_then(from_millis)
        && at - EXPIRY_MARGIN <= now
    {
        return Err(Problem::Expired(at));
    }
    Ok(token)
}

/// One poll: the token read from `credentials` now, the GET to `url`, the answer read.
pub fn poll(credentials: &Path, url: &str, now: DateTime<Utc>) -> Result<Reading, Problem> {
    let url = http::Url::parse(url).ok_or_else(|| Problem::Network(format!("not a URL: {url}")))?;
    let headers = {
        let token = read_token(credentials, now)?;
        headers(&token)
    };
    let response = http::get(&url, &headers, &agent(), http::TIMEOUT).map_err(Problem::Network)?;
    drop(headers);
    if !(200..300).contains(&response.status) {
        return Err(Problem::Status {
            code: response.status,
            retry_after_s: response.retry_after.map(|wait| wait.as_secs()),
            message: error_message(&response.body),
        });
    }
    parse(&response.body)
}

/// The request's headers, the token among them.
fn headers(token: &Secret) -> Secret {
    let parts = [
        "Authorization: Bearer ",
        token.expose(),
        "\r\nanthropic-beta: ",
        BETA,
        "\r\nAccept: application/json\r\n",
    ];
    // One allocation of the full size, so that no partial copy is left behind by a regrowth.
    let mut text = String::with_capacity(parts.iter().map(|part| part.len()).sum());
    for part in parts {
        text.push_str(part);
    }
    Secret(text)
}

/// The tool's own name in its requests.
fn agent() -> String {
    format!("clauderoadmapwrangler/{}", env!("CARGO_PKG_VERSION"))
}

/// An error answer's `error.type` and `error.message`, cut short.
fn error_message(body: &[u8]) -> Option<String> {
    let value: Value = serde_json::from_slice(body).ok()?;
    let error = value.get("error")?;
    let kind = error.get("type").and_then(Value::as_str);
    let message = error.get("message").and_then(Value::as_str);
    let text = match (kind, message) {
        (Some(kind), Some(message)) => format!("{kind}: {message}"),
        (Some(text), None) | (None, Some(text)) => text.to_owned(),
        (None, None) => return None,
    };
    Some(cut(&text, MESSAGE_CAP))
}

/// One limit: its share used, in percent, and when its window resets.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Limit {
    pub percent: f64,
    #[serde(with = "rfc3339", default)]
    pub resets_at: Option<DateTime<Utc>>,
    /// The endpoint's `is_active`, when it gave one.
    #[serde(default)]
    pub active: Option<bool>,
    /// The endpoint's `severity`, when it gave one.
    #[serde(default)]
    pub severity: Option<String>,
}

impl Limit {
    /// Its window has not reset yet: the share still counts. The endpoint's `is_active` says
    /// nothing of that: on 2026-11-05 it gave the weekly limit `true` and the Fable-scoped one
    /// `false`, both resetting three days later (EP-18), so the alerts and the downshift ask this.
    pub fn before_reset(&self, now: DateTime<Utc>) -> bool {
        self.resets_at.is_none_or(|at| at > now)
    }

    /// Its window has not reset yet, and the endpoint did not call it inactive: the five-hour
    /// figure's test, shown only while its window is open.
    pub fn is_current(&self, now: DateTime<Utc>) -> bool {
        self.active != Some(false) && self.before_reset(now)
    }

    /// The same share with the same reset, within a minute.
    fn same(&self, other: &Limit) -> bool {
        (self.percent - other.percent).abs() < 0.05 && same_reset(self.resets_at, other.resets_at)
    }
}

/// A scoped weekly limit, by its model's name.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Scoped {
    pub name: String,
    pub limit: Limit,
}

/// One surface's share of the week's use.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Surface {
    pub name: String,
    pub percent: f64,
}

/// Extra usage, as far as the endpoint gave it.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Extra {
    pub enabled: Option<bool>,
    pub percent: Option<f64>,
    pub used: Option<f64>,
    pub limit: Option<f64>,
}

/// What one answer of the endpoint said.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Reading {
    pub weekly: Option<Limit>,
    #[serde(default)]
    pub scoped: Vec<Scoped>,
    pub five_hour: Option<Limit>,
    #[serde(default)]
    pub surfaces: Vec<Surface>,
    pub extra: Option<Extra>,
    /// The `limits[]` entries of a kind the parser does not know, each named by its kind, so
    /// that a limit the endpoint adds or brings back still shows.
    #[serde(default)]
    pub other: Vec<Scoped>,
    /// The answer's shape: its keys and value kinds two levels down and the kinds of its
    /// `limits[]`, never a number or a text (`crw usage` prints it).
    #[serde(default)]
    pub shape: String,
}

impl Reading {
    /// The Fable-scoped weekly limit.
    pub fn fable(&self) -> Option<&Limit> {
        self.scoped
            .iter()
            .find(|scoped| scoped.name.to_lowercase().contains("fable"))
            .map(|scoped| &scoped.limit)
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap_or_default()
    }

    pub fn from_json(text: &str) -> Option<Reading> {
        serde_json::from_str(text).ok()
    }
}

/// Reads an answer of the endpoint. Fields it does not know are passed over, and a `limits[]`
/// kind it does not know is kept by its name; an answer with neither a weekly nor a scoped limit
/// is a schema change, named with the body's shape.
pub fn parse(body: &[u8]) -> Result<Reading, Problem> {
    let value: Value = serde_json::from_slice(body).map_err(|e| {
        Problem::Schema(format!(
            "not JSON ({:?} at line {}, column {})",
            e.classify(),
            e.line(),
            e.column()
        ))
    })?;
    let Some(object) = value.as_object() else {
        return Err(Problem::Schema(format!("the body is {}", shape(&value))));
    };
    let mut weekly = None;
    let mut scoped = Vec::new();
    let mut other = Vec::new();
    let mut kinds = Vec::new();
    for entry in object
        .get("limits")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
    {
        let kind = entry.get("kind").and_then(Value::as_str).map(str::trim);
        kinds.push(kind.unwrap_or("?"));
        let Some(limit) = limit_of(entry) else {
            continue;
        };
        match kind {
            Some("weekly_all") if weekly.is_none() => weekly = Some(limit),
            Some("weekly_scoped") => {
                let name = entry
                    .pointer("/scope/model/display_name")
                    .and_then(Value::as_str)
                    .unwrap_or("Scoped")
                    .trim();
                scoped.push(Scoped {
                    name: name.to_owned(),
                    limit,
                });
            }
            Some(kind) if !kind.is_empty() => other.push(Scoped {
                name: kind.to_owned(),
                limit,
            }),
            _ => {}
        }
    }
    // The limits' kinds first, so that a cut never takes them.
    let kinds = if kinds.is_empty() {
        "none".to_owned()
    } else {
        kinds.join(", ")
    };
    let outline = cut(
        &format!("the limits' kinds: {kinds}; {}", shape(&value)),
        SHAPE_CAP,
    );
    let weekly = weekly.or_else(|| object.get("seven_day").and_then(limit_of));
    if weekly.is_none() && scoped.is_empty() {
        return Err(Problem::Schema(format!(
            "no weekly limit in {}",
            shape(&value)
        )));
    }
    let surfaces = object
        .get("seven_day_breakdown")
        .and_then(|breakdown| breakdown.get("rows"))
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|row| {
            let name = first_str(row, &["label", "display_name", "name", "surface"])?;
            let percent =
                first_number(row, &["percent", "utilization", "share", "used_percentage"])?;
            Some(Surface {
                name: name.trim().to_owned(),
                percent,
            })
        })
        .collect();
    let extra = object
        .get("extra_usage")
        .filter(|extra| extra.is_object())
        .map(|extra| Extra {
            enabled: first_bool(extra, &["is_enabled", "enabled"]),
            percent: first_number(extra, &["utilization", "percent"]),
            used: first_number(extra, &["used_credits", "used"]),
            limit: first_number(extra, &["monthly_limit", "limit"]),
        });
    Ok(Reading {
        weekly,
        scoped,
        five_hour: object.get("five_hour").and_then(limit_of),
        surfaces,
        extra,
        other,
        shape: outline,
    })
}

/// A limit's share and reset from one entry: `percent`, else `utilization` or `used_percentage`.
fn limit_of(value: &Value) -> Option<Limit> {
    let percent = first_number(value, &["percent", "utilization", "used_percentage"])?;
    Some(Limit {
        percent,
        resets_at: value.get("resets_at").and_then(time_of),
        active: value.get("is_active").and_then(Value::as_bool),
        severity: value
            .get("severity")
            .and_then(Value::as_str)
            .map(str::to_owned),
    })
}

fn first_number(value: &Value, keys: &[&str]) -> Option<f64> {
    keys.iter()
        .find_map(|key| value.get(*key).and_then(Value::as_f64))
        .filter(|number| number.is_finite())
}

fn first_str<'a>(value: &'a Value, keys: &[&str]) -> Option<&'a str> {
    keys.iter()
        .find_map(|key| value.get(*key).and_then(Value::as_str))
        .filter(|text| !text.trim().is_empty())
}

fn first_bool(value: &Value, keys: &[&str]) -> Option<bool> {
    keys.iter()
        .find_map(|key| value.get(*key).and_then(Value::as_bool))
}

/// A time given as RFC 3339 text, or as seconds or milliseconds since the epoch.
fn time_of(value: &Value) -> Option<DateTime<Utc>> {
    match value {
        Value::String(text) => rfc3339::parse(text),
        Value::Number(number) => {
            let number = number.as_f64()?;
            if number > 1e11 {
                from_millis(number)
            } else {
                from_seconds(number)
            }
        }
        _ => None,
    }
}

fn from_millis(millis: f64) -> Option<DateTime<Utc>> {
    millis
        .is_finite()
        .then(|| DateTime::from_timestamp_millis(millis as i64))
        .flatten()
}

fn from_seconds(seconds: f64) -> Option<DateTime<Utc>> {
    seconds
        .is_finite()
        .then(|| DateTime::from_timestamp(seconds as i64, 0))
        .flatten()
}

/// A value's shape in a few words: an object's keys with their kinds two levels down, an array's
/// length and its first element's shape, never a value.
fn shape(value: &Value) -> String {
    cut(&shape_at(value, 2), SHAPE_CAP)
}

fn shape_at(value: &Value, depth: usize) -> String {
    match value {
        Value::Null => "null".to_owned(),
        Value::Bool(_) => "a bool".to_owned(),
        Value::Number(_) => "a number".to_owned(),
        Value::String(_) => "a string".to_owned(),
        Value::Array(items) => match items.first() {
            None => "[]".to_owned(),
            Some(first) if depth > 0 => {
                format!("[{} × {}]", items.len(), shape_at(first, depth - 1))
            }
            Some(_) => format!("[{} items]", items.len()),
        },
        // The keys with a value first, then those that are null, by name: an answer carries many
        // null promotional fields, and a cut should fall on them.
        Value::Object(map) if depth > 0 => {
            let (nulls, set): (Vec<_>, Vec<_>) = map.iter().partition(|(_, value)| value.is_null());
            let mut text = set
                .iter()
                .map(|(key, value)| format!("{key}: {}", shape_at(value, depth - 1)))
                .collect::<Vec<_>>()
                .join(", ");
            if !nulls.is_empty() {
                let names: Vec<&str> = nulls.iter().map(|(key, _)| key.as_str()).collect();
                if !text.is_empty() {
                    text.push_str("; ");
                }
                text.push_str(&format!("null: {}", names.join(", ")));
            }
            format!("{{{text}}}")
        }
        Value::Object(map) => format!("{{{} keys}}", map.len()),
    }
}

fn cut(text: &str, max: usize) -> String {
    if text.chars().count() <= max {
        return text.to_owned();
    }
    let mut short: String = text.chars().take(max - 1).collect();
    short.push('…');
    short
}

/// Two resets the same within a minute; two unknown ones the same.
fn same_reset(a: Option<DateTime<Utc>>, b: Option<DateTime<Utc>>) -> bool {
    match (a, b) {
        (Some(a), Some(b)) => (a - b).abs() <= TimeDelta::minutes(1),
        (None, None) => true,
        _ => false,
    }
}

/// When the next poll is due: [`EVERY`] while the endpoint answers, the backoff while it fails.
#[derive(Debug, Clone)]
pub struct Schedule {
    due: DateTime<Utc>,
    last: Option<DateTime<Utc>>,
    failures: u32,
}

impl Schedule {
    /// The first poll comes [`FIRST`] after `now`.
    pub fn new(now: DateTime<Utc>) -> Self {
        Self {
            due: now + FIRST,
            last: None,
            failures: 0,
        }
    }

    pub fn due(&self) -> DateTime<Utc> {
        self.due
    }

    /// The failed polls in a row.
    pub fn failures(&self) -> u32 {
        self.failures
    }

    /// Sets the next poll after one at `at`: [`EVERY`] after a reading; after a failure of the
    /// endpoint's the backoff, or the longer wait it asked for; a minute after a problem on this
    /// machine, and [`EVERY`] while the switch is off.
    pub fn polled(&mut self, at: DateTime<Utc>, problem: Option<&Problem>) {
        self.last = Some(at);
        self.due = at
            + match problem {
                None => {
                    self.failures = 0;
                    EVERY
                }
                Some(Problem::Off) => EVERY,
                Some(problem) if problem.is_local() => LOCAL_RETRY,
                Some(problem) => {
                    self.failures += 1;
                    let wait = backoff(self.failures);
                    problem
                        .retry_after()
                        .map_or(wait, |asked| asked.clamp(wait, LONGEST_WAIT))
                }
            };
    }

    /// A session the tool launched has ended: the next poll comes at once, though no sooner than
    /// [`AFTER_END`] after the last. While the endpoint fails, its backoff stands.
    pub fn session_ended(&mut self, now: DateTime<Utc>) {
        if self.failures > 0 {
            return;
        }
        let soonest = self.last.map_or(now, |last| (last + AFTER_END).max(now));
        self.due = self.due.min(soonest);
    }

    /// The switch turned on: the next poll comes at once.
    pub fn now(&mut self, now: DateTime<Utc>) {
        self.failures = 0;
        self.due = now;
    }
}

fn backoff(failures: u32) -> TimeDelta {
    usize::try_from(failures.saturating_sub(1))
        .ok()
        .and_then(|at| BACKOFF_MINUTES.get(at))
        .map_or(BACKOFF_CAP, |minutes| TimeDelta::minutes(*minutes))
}

/// A session's status-line figures (EP-12), as the meter's second series.
#[derive(Debug, Clone, PartialEq)]
pub struct Line {
    pub at: DateTime<Utc>,
    pub weekly: Option<Limit>,
    pub five_hour: Option<Limit>,
}

impl Line {
    /// The figures of a status line; none when it gave no rate limits (before the session's
    /// first answer, or on a plan without them).
    pub fn of(status: &StatusLine, at: DateTime<Utc>) -> Option<Line> {
        let limits = status.rate_limits.as_ref()?;
        let weekly = limits.seven_day.as_ref().and_then(window_limit);
        let five_hour = limits.five_hour.as_ref().and_then(window_limit);
        (weekly.is_some() || five_hour.is_some()).then_some(Line {
            at,
            weekly,
            five_hour,
        })
    }

    fn same_figures(&self, other: &Line) -> bool {
        let same = |a: &Option<Limit>, b: &Option<Limit>| match (a, b) {
            (Some(a), Some(b)) => a.same(b),
            (None, None) => true,
            _ => false,
        };
        same(&self.weekly, &other.weekly) && same(&self.five_hour, &other.five_hour)
    }
}

fn window_limit(window: &signals::Window) -> Option<Limit> {
    Some(Limit {
        percent: window.used_percentage.filter(|p| p.is_finite())?,
        resets_at: window.resets_at.and_then(from_seconds),
        active: None,
        severity: None,
    })
}

/// Where a figure came from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Source {
    Endpoint,
    StatusLine,
}

impl Source {
    /// The ledger's word (`usage_sample.source`).
    pub fn as_str(self) -> &'static str {
        match self {
            Source::Endpoint => "endpoint",
            Source::StatusLine => "statusline",
        }
    }
}

/// One figure as the meter shows it: the limit, its source and the time it was read.
#[derive(Debug, Clone, PartialEq)]
pub struct Figure {
    pub limit: Limit,
    pub source: Source,
    pub at: DateTime<Utc>,
}

/// What the strip and the tile show.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Figures {
    pub weekly: Option<Figure>,
    pub fable: Option<Figure>,
    /// While a five-hour window is open.
    pub five_hour: Option<Figure>,
    /// Why the endpoint's figures do not stand, when they do not.
    pub degraded: Option<String>,
}

/// A change of the meter's state, for the ledger's event log.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Shift {
    Degraded(String),
    Recovered,
}

/// The endpoint's last reading with its time.
#[derive(Debug, Clone, PartialEq)]
pub struct Fetched {
    pub at: DateTime<Utc>,
    pub reading: Reading,
}

/// The meter's state: the endpoint's last reading and how its last poll went, the newest
/// status-line figures of each session, and the notifications sent.
#[derive(Debug, Clone, Default)]
pub struct Gauge {
    pub fetched: Option<Fetched>,
    /// Why the last poll failed, and when; none once one succeeds.
    pub problem: Option<(Problem, DateTime<Utc>)>,
    pub lines: BTreeMap<String, Line>,
    pub alerts: Alerts,
}

impl Gauge {
    /// Takes in a poll's outcome; returns the change of state worth a line of the log.
    pub fn polled(
        &mut self,
        at: DateTime<Utc>,
        outcome: Result<Reading, Problem>,
    ) -> Option<Shift> {
        match outcome {
            Ok(reading) => {
                self.fetched = Some(Fetched { at, reading });
                self.problem.take().map(|_| Shift::Recovered)
            }
            Err(problem) => {
                let words = problem.words();
                let again = self
                    .problem
                    .as_ref()
                    .is_some_and(|(before, _)| before.words() == words);
                self.problem = Some((problem, at));
                (!again).then_some(Shift::Degraded(words))
            }
        }
    }

    /// Takes in a session's status-line figures; true when they differ from its last ones, so
    /// that a sample is due.
    pub fn line(&mut self, session_id: &str, line: Line) -> bool {
        let changed = self
            .lines
            .get(session_id)
            .is_none_or(|before| !before.same_figures(&line));
        self.lines.insert(session_id.to_owned(), line);
        changed
    }

    /// The newest status-line figure of one kind across the sessions.
    fn newest_line(&self, pick: impl Fn(&Line) -> Option<&Limit>) -> Option<Figure> {
        self.lines
            .values()
            .filter_map(|line| {
                pick(line).map(|limit| Figure {
                    limit: limit.clone(),
                    source: Source::StatusLine,
                    at: line.at,
                })
            })
            .max_by_key(|figure| figure.at)
    }

    /// The figures as of `now`. The endpoint's stand while its last poll succeeded and its
    /// reading is fresh; otherwise each figure is the newest of its last reading's and the
    /// status lines' (Fable from the reading alone), and a failed poll marks them degraded.
    pub fn figures(&self, now: DateTime<Utc>) -> Figures {
        let endpoint = |pick: &dyn Fn(&Reading) -> Option<&Limit>| {
            self.fetched.as_ref().and_then(|fetched| {
                pick(&fetched.reading).map(|limit| Figure {
                    limit: limit.clone(),
                    source: Source::Endpoint,
                    at: fetched.at,
                })
            })
        };
        let standing = self.problem.is_none()
            && self
                .fetched
                .as_ref()
                .is_some_and(|fetched| now - fetched.at <= FRESH_FOR);
        let newest = |a: Option<Figure>, b: Option<Figure>| match (a, b) {
            (Some(a), Some(b)) => Some(if b.at > a.at { b } else { a }),
            (a, b) => a.or(b),
        };
        let weekly = endpoint(&|reading| reading.weekly.as_ref());
        let five_hour = endpoint(&|reading| reading.five_hour.as_ref());
        let (weekly, five_hour) = if standing {
            (weekly, five_hour)
        } else {
            (
                newest(weekly, self.newest_line(|line| line.weekly.as_ref())),
                newest(five_hour, self.newest_line(|line| line.five_hour.as_ref())),
            )
        };
        Figures {
            weekly,
            fable: endpoint(&|reading| reading.fable()),
            five_hour: five_hour.filter(|figure| figure.limit.is_current(now)),
            degraded: self.problem.as_ref().map(|(problem, _)| problem.words()),
        }
    }

    /// The marks the weekly and Fable figures reach as of `now` and have not been notified in
    /// their window: a figure counts until its reset, whatever the endpoint's `is_active` says
    /// (EP-18).
    pub fn crossings(&mut self, now: DateTime<Utc>) -> Vec<Crossing> {
        let figures = self.figures(now);
        [
            (Which::Weekly, figures.weekly),
            (Which::Fable, figures.fable),
        ]
        .into_iter()
        .filter_map(|(which, figure)| {
            let figure = figure.filter(|figure| figure.limit.before_reset(now))?;
            self.alerts.check(which, &figure.limit)
        })
        .collect()
    }
}

/// The limits notified.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Which {
    Weekly,
    Fable,
}

impl Which {
    pub fn name(self) -> &'static str {
        match self {
            Which::Weekly => "Weekly",
            Which::Fable => "Fable",
        }
    }
}

/// A mark reached: which limit, the mark, its share now and its window's reset.
#[derive(Debug, Clone, PartialEq)]
pub struct Crossing {
    pub which: Which,
    pub mark: f64,
    pub percent: f64,
    pub resets_at: Option<DateTime<Utc>>,
}

/// The highest mark notified in a limit's window.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Alerted {
    pub mark: f64,
    #[serde(with = "rfc3339", default)]
    pub resets_at: Option<DateTime<Utc>>,
}

/// What was notified of the weekly and Fable limits; kept in the ledger's cache, so that a
/// restart does not notify a mark again.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Alerts {
    #[serde(default)]
    pub weekly: Option<Alerted>,
    #[serde(default)]
    pub fable: Option<Alerted>,
}

impl Alerts {
    /// The mark `limit` reaches, if it has not been notified in this window: [`ALARM`] over
    /// [`WARN`], once each. A share below [`WARN`] clears the limit's record: its window is a
    /// new one.
    pub fn check(&mut self, which: Which, limit: &Limit) -> Option<Crossing> {
        let slot = match which {
            Which::Weekly => &mut self.weekly,
            Which::Fable => &mut self.fable,
        };
        let mark = if limit.percent >= ALARM {
            ALARM
        } else if limit.percent >= WARN {
            WARN
        } else {
            *slot = None;
            return None;
        };
        let sent = slot
            .as_ref()
            .filter(|alerted| same_reset(alerted.resets_at, limit.resets_at))
            .map(|alerted| alerted.mark);
        if sent.is_some_and(|sent| sent >= mark) {
            return None;
        }
        *slot = Some(Alerted {
            mark,
            resets_at: limit.resets_at,
        });
        Some(Crossing {
            which,
            mark,
            percent: limit.percent,
            resets_at: limit.resets_at,
        })
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap_or_default()
    }

    pub fn from_json(text: &str) -> Option<Alerts> {
        serde_json::from_str(text).ok()
    }
}

/// The ledger row of an endpoint's reading.
pub fn endpoint_sample(reading: &Reading, at: DateTime<Utc>) -> Sample {
    let weekly = reading.weekly.as_ref();
    let fable = reading.fable();
    let five_hour = reading.five_hour.as_ref();
    Sample {
        taken_at: at,
        source: Source::Endpoint.as_str().to_owned(),
        session_id: None,
        weekly_percent: weekly.map(|limit| limit.percent),
        weekly_resets_at: weekly.and_then(|limit| limit.resets_at),
        fable_percent: fable.map(|limit| limit.percent),
        fable_resets_at: fable.and_then(|limit| limit.resets_at),
        five_hour_percent: five_hour.map(|limit| limit.percent),
        five_hour_resets_at: five_hour.and_then(|limit| limit.resets_at),
    }
}

/// The ledger row of a session's status-line figures.
pub fn line_sample(session_id: &str, line: &Line) -> Sample {
    Sample {
        taken_at: line.at,
        source: Source::StatusLine.as_str().to_owned(),
        session_id: Some(session_id.to_owned()),
        weekly_percent: line.weekly.as_ref().map(|limit| limit.percent),
        weekly_resets_at: line.weekly.as_ref().and_then(|limit| limit.resets_at),
        fable_percent: None,
        fable_resets_at: None,
        five_hour_percent: line.five_hour.as_ref().map(|limit| limit.percent),
        five_hour_resets_at: line.five_hour.as_ref().and_then(|limit| limit.resets_at),
    }
}

/// The time left until `at`, in a word or two: `2 d 4 h`, `3 h 12 min`, `12 min`, `now`.
pub fn countdown(at: DateTime<Utc>, now: DateTime<Utc>) -> String {
    let left = at - now;
    let minutes = left.num_minutes();
    if minutes < 1 {
        return "now".to_owned();
    }
    let (days, hours, minutes) = (minutes / 1440, minutes / 60 % 24, minutes % 60);
    match (days, hours) {
        (0, 0) => format!("{minutes} min"),
        (0, _) => format!("{hours} h {minutes} min"),
        _ => format!("{days} d {hours} h"),
    }
}

/// A share in percent as the meter writes it: `41 %`, `0.5 %`.
pub fn percent(value: f64) -> String {
    if value > 0.0 && value < 1.0 {
        format!("{value:.1} %")
    } else {
        format!("{value:.0} %")
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::io::{Read, Write};
    use std::net::TcpListener;
    use std::thread;

    use super::*;
    use crate::files::tests::Scratch;

    fn at(text: &str) -> DateTime<Utc> {
        rfc3339::parse(text).unwrap()
    }

    /// A body shaped like the probe's of 2026-10-31 (planning record 2.3), with synthetic numbers.
    const BODY: &str = r#"{
        "five_hour": {"utilization": 23.0, "resets_at": "2026-11-05T03:00:00.000000+00:00"},
        "seven_day": {"utilization": 41.0, "resets_at": "2026-11-07T19:00:00+00:00"},
        "limits": [
            {"kind": "weekly_all", "percent": 41.0, "severity": "normal",
             "resets_at": "2026-11-07T19:00:00+00:00", "scope": null, "is_active": true},
            {"kind": "weekly_scoped", "percent": 63.5, "severity": "warning",
             "resets_at": "2026-11-07T19:00:00+00:00",
             "scope": {"model": {"display_name": "Fable"}}, "is_active": true},
            {"kind": "something_new", "percent": 3.0}
        ],
        "seven_day_breakdown": {"rows": [
            {"label": "Claude Code", "percent": 35.0}, {"label": "Chats", "percent": 4.0},
            {"label": "Cowork", "percent": 1.5}, {"label": "Other", "percent": 0.5},
            {"label": "Unnumbered"}
        ]},
        "weekly_scoped_shares": [{"name": "Fable", "share": 0.4}],
        "extra_usage": {"is_enabled": false, "monthly_limit": null, "used_credits": null,
                        "utilization": null},
        "spend": null,
        "a_promotion": null
    }"#;

    fn credentials(scratch: &Scratch, text: &str) -> PathBuf {
        let path = scratch.0.join(".credentials.json");
        fs::write(&path, text).unwrap();
        path
    }

    #[test]
    fn the_token_is_read_at_poll_time_and_never_printed() {
        let scratch = Scratch::new("usage-token");
        let now = at("2026-11-04T12:00:00Z");
        let path = credentials(
            &scratch,
            r#"{"claudeAiOauth": {"accessToken": "synthetic-access", "refreshToken": "synthetic-refresh",
                "expiresAt": 1793880000000, "scopes": ["user:inference", "user:profile"]}}"#,
        );
        let token = read_token(&path, now).unwrap();
        assert_eq!(token.expose(), "synthetic-access");
        assert_eq!(format!("{token:?}"), "Secret([redacted])");
        let headers = headers(&token);
        assert!(
            headers
                .expose()
                .starts_with("Authorization: Bearer synthetic-access\r\n")
        );
        assert!(
            headers
                .expose()
                .contains("anthropic-beta: oauth-2025-04-20\r\n")
        );
        assert!(!format!("{headers:?}").contains("synthetic"));
    }

    #[test]
    fn a_missing_unreadable_empty_or_expired_token_is_a_problem_of_its_own() {
        let scratch = Scratch::new("usage-token-problems");
        let now = at("2026-11-04T12:00:00Z");
        assert_eq!(
            read_token(&scratch.0.join("absent.json"), now).unwrap_err(),
            Problem::NoCredentials
        );
        let wrong = credentials(&scratch, r#"{"claudeAiOauth": {"accessToken": 12345}}"#);
        let Problem::Credentials(why) = read_token(&wrong, now).unwrap_err() else {
            panic!("a token that is a number");
        };
        assert!(!why.contains("12345"), "{why}");
        let no_sign_in = credentials(&scratch, r#"{"mcpOAuth": {}}"#);
        assert_eq!(read_token(&no_sign_in, now).unwrap_err(), Problem::NoToken);
        let blank = credentials(&scratch, r#"{"claudeAiOauth": {"accessToken": "  "}}"#);
        assert_eq!(read_token(&blank, now).unwrap_err(), Problem::NoToken);
        // 2026-11-04T11:00:00Z, an hour before `now`.
        let expired = credentials(
            &scratch,
            r#"{"claudeAiOauth": {"accessToken": "synthetic", "expiresAt": 1793790000000}}"#,
        );
        let problem = read_token(&expired, now).unwrap_err();
        assert_eq!(problem, Problem::Expired(at("2026-11-04T11:00:00Z")));
        assert!(problem.is_local());
        assert!(problem.words().contains("renews it"), "{}", problem.words());
    }

    #[test]
    fn an_answer_reads_its_limits_breakdown_and_extra_usage() {
        let reading = parse(BODY.as_bytes()).unwrap();
        let weekly = reading.weekly.as_ref().unwrap();
        assert_eq!(weekly.percent, 41.0);
        assert_eq!(weekly.resets_at, Some(at("2026-11-07T19:00:00Z")));
        assert_eq!(weekly.active, Some(true));
        let fable = reading.fable().unwrap();
        assert_eq!(
            (fable.percent, fable.severity.as_deref()),
            (63.5, Some("warning"))
        );
        let five = reading.five_hour.as_ref().unwrap();
        assert_eq!(
            (five.percent, five.resets_at),
            (23.0, Some(at("2026-11-05T03:00:00Z")))
        );
        let surfaces: Vec<(&str, f64)> = reading
            .surfaces
            .iter()
            .map(|surface| (surface.name.as_str(), surface.percent))
            .collect();
        assert_eq!(
            surfaces,
            [
                ("Claude Code", 35.0),
                ("Chats", 4.0),
                ("Cowork", 1.5),
                ("Other", 0.5)
            ]
        );
        assert_eq!(reading.extra.as_ref().unwrap().enabled, Some(false));
        // A kind the parser does not know is kept by its name, so a new limit still shows.
        assert_eq!(reading.other.len(), 1);
        assert_eq!(
            (
                reading.other[0].name.as_str(),
                reading.other[0].limit.percent
            ),
            ("something_new", 3.0)
        );
        // The shape names keys, kinds and the limits' kinds, never a number or a text.
        assert!(
            reading.shape.starts_with(
                "the limits' kinds: weekly_all, weekly_scoped, something_new; \
                 {five_hour: {utilization: a number, resets_at: a string}, "
            ),
            "{}",
            reading.shape
        );
        assert!(
            reading.shape.ends_with("; null: spend, a_promotion}"),
            "{}",
            reading.shape
        );
        assert!(!reading.shape.contains("41") && !reading.shape.contains("Fable"));
        assert_eq!(Reading::from_json(&reading.to_json()), Some(reading));
    }

    #[test]
    fn an_older_answer_and_a_closed_five_hour_window_still_read() {
        let reading = parse(
            br#"{"five_hour": null, "seven_day": {"utilization": 12.5, "resets_at": 1794078000},
                 "seven_day_opus": null}"#,
        )
        .unwrap();
        assert_eq!(reading.weekly.as_ref().unwrap().percent, 12.5);
        assert_eq!(
            reading.weekly.as_ref().unwrap().resets_at,
            Some(at("2026-11-07T19:00:00Z"))
        );
        assert_eq!(reading.five_hour, None);
        assert_eq!(reading.fable(), None);
        assert_eq!(
            reading.shape,
            "the limits' kinds: none; {seven_day: {utilization: a number, resets_at: a number}; \
             null: five_hour, seven_day_opus}"
        );
        // Many null keys are cut before the keys with a value.
        let mut body = String::from(r#"{"seven_day": {"utilization": 1.0}"#);
        for n in 0..80 {
            body.push_str(&format!(r#", "promotion_{n}": null"#));
        }
        body.push('}');
        let shape = parse(body.as_bytes()).unwrap().shape;
        assert!(
            shape.starts_with(
                "the limits' kinds: none; {seven_day: {utilization: a number}; null: "
            ),
            "{shape}"
        );
        assert!(
            shape.ends_with('…') && shape.chars().count() == SHAPE_CAP,
            "{shape}"
        );
        // A reading cached by a build without the shape still reads.
        let older = r#"{"weekly": {"percent": 12.5}, "five_hour": null, "extra": null}"#;
        assert_eq!(Reading::from_json(older).unwrap().shape, "");
    }

    #[test]
    fn a_changed_answer_is_named_by_its_shape_without_its_values() {
        let Problem::Schema(why) =
            parse(br#"{"usage": {"weekly": 41, "label": "secret-ish words"}, "items": [1, 2]}"#)
                .unwrap_err()
        else {
            panic!("a schema change");
        };
        assert_eq!(
            why,
            "no weekly limit in {usage: {weekly: a number, label: a string}, items: [2 × a number]}"
        );
        assert!(matches!(parse(b"<html>").unwrap_err(), Problem::Schema(_)));
        let Problem::Schema(why) = parse(b"[]").unwrap_err() else {
            panic!("an array");
        };
        assert_eq!(why, "the body is []");
    }

    /// A one-request HTTP server on 127.0.0.1 answering `answer`; the request it read comes back
    /// through the join handle. Never the endpoint (D-29).
    fn serve(answer: String) -> (String, thread::JoinHandle<String>) {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let url = format!("http://{}/api/oauth/usage", listener.local_addr().unwrap());
        let handle = thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            let mut request = Vec::new();
            let mut chunk = [0u8; 1024];
            while !request.windows(4).any(|w| w == b"\r\n\r\n") {
                let n = stream.read(&mut chunk).unwrap();
                if n == 0 {
                    break;
                }
                request.extend_from_slice(&chunk[..n]);
            }
            stream.write_all(answer.as_bytes()).unwrap();
            String::from_utf8_lossy(&request).into_owned()
        });
        (url, handle)
    }

    fn answer(status: &str, extra: &str, body: &str) -> String {
        format!(
            "HTTP/1.1 {status}\r\nContent-Type: application/json\r\n{extra}Content-Length: {}\r\n\
             Connection: close\r\n\r\n{body}",
            body.len()
        )
    }

    #[cfg(windows)]
    #[test]
    fn a_poll_sends_the_token_and_reads_the_answer_through_winhttp() {
        let scratch = Scratch::new("usage-poll");
        let now = at("2026-11-04T12:00:00Z");
        let path = credentials(
            &scratch,
            r#"{"claudeAiOauth": {"accessToken": "synthetic-access", "expiresAt": 1793880000000}}"#,
        );
        let (url, server) = serve(answer("200 OK", "", BODY));
        let reading = poll(&path, &url, now).unwrap();
        assert_eq!(reading.fable().unwrap().percent, 63.5);
        let request = server.join().unwrap().to_lowercase();
        assert!(
            request.starts_with("get /api/oauth/usage http/1.1\r\n"),
            "{request}"
        );
        assert!(
            request.contains("\r\nauthorization: bearer synthetic-access\r\n"),
            "{request}"
        );
        assert!(
            request.contains("\r\nanthropic-beta: oauth-2025-04-20\r\n"),
            "{request}"
        );
        assert!(
            request.contains("\r\nuser-agent: clauderoadmapwrangler/"),
            "{request}"
        );
    }

    #[cfg(windows)]
    #[test]
    fn a_refusal_or_a_wait_reads_its_status_message_and_retry_after() {
        let scratch = Scratch::new("usage-poll-refused");
        let now = at("2026-11-04T12:00:00Z");
        let path = credentials(
            &scratch,
            r#"{"claudeAiOauth": {"accessToken": "synthetic"}}"#,
        );
        let (url, server) = serve(answer(
            "429 Too Many Requests",
            "Retry-After: 120\r\n",
            r#"{"error": {"type": "rate_limit_error", "message": "Slow down"}}"#,
        ));
        let problem = poll(&path, &url, now).unwrap_err();
        server.join().unwrap();
        assert_eq!(
            problem,
            Problem::Status {
                code: 429,
                retry_after_s: Some(120),
                message: Some("rate_limit_error: Slow down".to_owned()),
            }
        );
        assert_eq!(problem.retry_after(), Some(TimeDelta::minutes(2)));
        let (url, server) = serve(answer("401 Unauthorized", "", "not json"));
        let problem = poll(&path, &url, now).unwrap_err();
        server.join().unwrap();
        assert!(
            problem
                .words()
                .starts_with("the endpoint refused the token (HTTP 401)")
        );
        // Nothing listens on a port just let go: a network problem, in words.
        let port = TcpListener::bind("127.0.0.1:0")
            .unwrap()
            .local_addr()
            .unwrap()
            .port();
        let url = format!("http://127.0.0.1:{port}/api/oauth/usage");
        assert!(matches!(
            poll(&path, &url, now).unwrap_err(),
            Problem::Network(_)
        ));
    }

    #[test]
    fn the_schedule_polls_every_five_minutes_and_backs_off_while_failing() {
        let start = at("2026-11-04T12:00:00Z");
        let mut schedule = Schedule::new(start);
        assert_eq!(schedule.due(), start + FIRST);
        schedule.polled(start, None);
        assert_eq!(schedule.due(), start + EVERY);
        let network = Problem::Network("no route".to_owned());
        let mut waits = Vec::new();
        let mut now = start;
        for _ in 0..7 {
            schedule.polled(now, Some(&network));
            waits.push((schedule.due() - now).num_minutes());
            now = schedule.due();
        }
        assert_eq!(waits, [1, 2, 4, 8, 16, 30, 30]);
        // A session's end does not cut a backoff short.
        schedule.polled(now, Some(&network));
        schedule.session_ended(now + TimeDelta::minutes(2));
        assert_eq!(schedule.due(), now + TimeDelta::minutes(30));
        // A 429's longer wait stands over the backoff.
        let mut asked = Schedule::new(start);
        let wait = Problem::Status {
            code: 429,
            retry_after_s: Some(3600),
            message: None,
        };
        asked.polled(start, Some(&wait));
        assert_eq!(asked.due(), start + TimeDelta::hours(1));
        schedule.polled(now, None);
        assert_eq!((schedule.failures(), schedule.due()), (0, now + EVERY));
        // A problem on this machine is looked at again in a minute, never backed off.
        schedule.polled(now, Some(&Problem::NoCredentials));
        assert_eq!(
            (schedule.failures(), schedule.due()),
            (0, now + LOCAL_RETRY)
        );
        schedule.polled(now, Some(&Problem::Off));
        assert_eq!(schedule.due(), now + EVERY);
    }

    #[test]
    fn a_session_end_brings_the_poll_forward_but_not_within_a_minute() {
        let start = at("2026-11-04T12:00:00Z");
        let mut schedule = Schedule::new(start);
        schedule.polled(start, None);
        schedule.session_ended(start + TimeDelta::seconds(20));
        assert_eq!(schedule.due(), start + AFTER_END);
        schedule.polled(start + AFTER_END, None);
        let later = start + TimeDelta::minutes(4);
        schedule.session_ended(later);
        assert_eq!(schedule.due(), later);
    }

    fn line(at_text: &str, weekly: f64) -> Line {
        Line {
            at: at(at_text),
            weekly: Some(Limit {
                percent: weekly,
                resets_at: Some(at("2026-11-07T19:00:00Z")),
                active: None,
                severity: None,
            }),
            five_hour: Some(Limit {
                percent: 10.0,
                resets_at: Some(at("2026-11-04T14:00:00Z")),
                active: None,
                severity: None,
            }),
        }
    }

    #[test]
    fn the_endpoint_stands_and_the_status_lines_fill_in_when_it_fails() {
        let now = at("2026-11-04T12:00:00Z");
        let mut gauge = Gauge::default();
        assert_eq!(gauge.figures(now), Figures::default());
        assert_eq!(gauge.polled(now, Ok(parse(BODY.as_bytes()).unwrap())), None);
        assert!(gauge.line("s-1", line("2026-11-04T12:01:00Z", 44.0)));
        assert!(!gauge.line("s-1", line("2026-11-04T12:01:30Z", 44.0)));
        let later = now + TimeDelta::minutes(2);
        let figures = gauge.figures(later);
        assert_eq!(figures.weekly.as_ref().unwrap().source, Source::Endpoint);
        assert_eq!(figures.weekly.unwrap().limit.percent, 41.0);
        assert_eq!(figures.degraded, None);
        // The endpoint fails: the newer status-line figures show, degraded; Fable stays the
        // endpoint's.
        let failed = gauge.polled(later, Err(Problem::Network("timed out".to_owned())));
        assert_eq!(
            failed,
            Some(Shift::Degraded(
                "the endpoint did not answer: timed out".to_owned()
            ))
        );
        assert_eq!(
            gauge.polled(later, Err(Problem::Network("timed out".to_owned()))),
            None
        );
        let figures = gauge.figures(later);
        let weekly = figures.weekly.unwrap();
        assert_eq!(
            (weekly.source, weekly.limit.percent),
            (Source::StatusLine, 44.0)
        );
        assert_eq!(figures.fable.unwrap().source, Source::Endpoint);
        assert_eq!(figures.five_hour.unwrap().limit.percent, 10.0);
        assert!(figures.degraded.unwrap().contains("timed out"));
        // Back again.
        assert_eq!(
            gauge.polled(later, Ok(parse(BODY.as_bytes()).unwrap())),
            Some(Shift::Recovered)
        );
        // A five-hour window past its reset is no longer shown.
        assert_eq!(gauge.figures(at("2026-11-05T04:00:00Z")).five_hour, None);
    }

    #[test]
    fn a_stale_reading_gives_way_to_newer_status_lines_without_the_degraded_mark() {
        let now = at("2026-11-04T12:00:00Z");
        let mut gauge = Gauge::default();
        gauge.polled(
            now - TimeDelta::hours(3),
            Ok(parse(BODY.as_bytes()).unwrap()),
        );
        gauge.line("s-1", line("2026-11-04T11:59:00Z", 47.0));
        let figures = gauge.figures(now);
        assert_eq!(figures.weekly.unwrap().limit.percent, 47.0);
        assert_eq!(figures.degraded, None);
    }

    fn limit(percent: f64, resets_at: &str) -> Limit {
        Limit {
            percent,
            resets_at: Some(at(resets_at)),
            active: Some(true),
            severity: None,
        }
    }

    #[test]
    fn each_mark_is_notified_once_per_window() {
        let mut alerts = Alerts::default();
        let week = "2026-11-07T19:00:00Z";
        assert_eq!(alerts.check(Which::Weekly, &limit(79.9, week)), None);
        let first = alerts.check(Which::Weekly, &limit(80.0, week)).unwrap();
        assert_eq!((first.which, first.mark), (Which::Weekly, WARN));
        assert_eq!(alerts.check(Which::Weekly, &limit(88.0, week)), None);
        assert_eq!(
            alerts
                .check(Which::Weekly, &limit(96.0, week))
                .unwrap()
                .mark,
            ALARM
        );
        assert_eq!(alerts.check(Which::Weekly, &limit(99.0, week)), None);
        // Kept across a restart.
        let mut restored = Alerts::from_json(&alerts.to_json()).unwrap();
        assert_eq!(restored.check(Which::Weekly, &limit(99.0, week)), None);
        // A new window notifies again; reached at once, the 95 alone.
        let next = "2026-11-14T19:00:00Z";
        assert_eq!(
            restored
                .check(Which::Weekly, &limit(97.0, next))
                .unwrap()
                .mark,
            ALARM
        );
        assert_eq!(restored.check(Which::Weekly, &limit(98.0, next)), None);
        // Fable is a limit of its own.
        assert_eq!(
            restored
                .check(Which::Fable, &limit(81.0, next))
                .unwrap()
                .which,
            Which::Fable
        );
        // A share back under the mark is a new window, even with no reset known.
        let mut unknown = Alerts::default();
        let no_reset = |percent| Limit {
            percent,
            resets_at: None,
            active: None,
            severity: None,
        };
        assert!(unknown.check(Which::Fable, &no_reset(85.0)).is_some());
        assert!(unknown.check(Which::Fable, &no_reset(86.0)).is_none());
        assert!(unknown.check(Which::Fable, &no_reset(2.0)).is_none());
        assert!(unknown.check(Which::Fable, &no_reset(85.0)).is_some());
    }

    #[test]
    fn the_gauge_notifies_its_standing_weekly_and_fable_figures() {
        let now = at("2026-11-04T12:00:00Z");
        let mut gauge = Gauge::default();
        let mut reading = parse(BODY.as_bytes()).unwrap();
        reading.weekly.as_mut().unwrap().percent = 82.0;
        reading.scoped[0].limit.percent = 95.5;
        gauge.polled(now, Ok(reading));
        let crossings: Vec<(Which, f64)> = gauge
            .crossings(now)
            .into_iter()
            .map(|crossing| (crossing.which, crossing.mark))
            .collect();
        assert_eq!(crossings, [(Which::Weekly, WARN), (Which::Fable, ALARM)]);
        assert!(gauge.crossings(now).is_empty());
        // The endpoint calls the Fable-scoped limit inactive while its window runs (seen
        // 2026-11-05): its marks are notified all the same.
        let mut inactive = Gauge::default();
        let mut reading = parse(BODY.as_bytes()).unwrap();
        reading.scoped[0].limit.percent = 83.0;
        reading.scoped[0].limit.active = Some(false);
        inactive.polled(now, Ok(reading));
        let marks: Vec<(Which, f64)> = inactive
            .crossings(now)
            .into_iter()
            .map(|crossing| (crossing.which, crossing.mark))
            .collect();
        assert_eq!(marks, [(Which::Fable, WARN)]);
        // A limit whose window has reset is not notified from an old reading.
        let mut stale = Gauge::default();
        let mut reading = parse(BODY.as_bytes()).unwrap();
        reading.weekly.as_mut().unwrap().percent = 90.0;
        stale.polled(now, Ok(reading));
        assert!(stale.crossings(at("2026-11-08T00:00:00Z")).is_empty());
    }

    #[test]
    fn samples_carry_the_endpoints_and_the_status_lines_figures() {
        let now = at("2026-11-04T12:00:00Z");
        let sample = endpoint_sample(&parse(BODY.as_bytes()).unwrap(), now);
        assert_eq!(sample.source, "endpoint");
        assert_eq!(
            (
                sample.weekly_percent,
                sample.fable_percent,
                sample.five_hour_percent
            ),
            (Some(41.0), Some(63.5), Some(23.0))
        );
        assert_eq!(sample.fable_resets_at, Some(at("2026-11-07T19:00:00Z")));
        let sample = line_sample("s-1", &line("2026-11-04T12:01:00Z", 44.0));
        assert_eq!(sample.source, "statusline");
        assert_eq!(sample.session_id.as_deref(), Some("s-1"));
        assert_eq!(
            (sample.weekly_percent, sample.fable_percent),
            (Some(44.0), None)
        );
    }

    #[test]
    fn a_status_line_gives_its_rate_limits_as_figures() {
        let status: StatusLine = serde_json::from_str(
            r#"{"session_id": "s-1", "rate_limits": {"five_hour": {"used_percentage": 23.5,
                "resets_at": 1793800800}, "seven_day": {"used_percentage": 41.2}}}"#,
        )
        .unwrap();
        let line = Line::of(&status, at("2026-11-04T12:00:00Z")).unwrap();
        assert_eq!(line.weekly.as_ref().unwrap().percent, 41.2);
        assert_eq!(line.weekly.as_ref().unwrap().resets_at, None);
        assert_eq!(
            line.five_hour.as_ref().unwrap().resets_at,
            Some(at("2026-11-04T14:00:00Z"))
        );
        assert_eq!(
            Line::of(&StatusLine::default(), at("2026-11-04T12:00:00Z")),
            None
        );
    }

    #[test]
    fn countdowns_and_percents_read_short() {
        let now = at("2026-11-04T12:00:00Z");
        assert_eq!(countdown(now + TimeDelta::seconds(30), now), "now");
        assert_eq!(countdown(now + TimeDelta::minutes(12), now), "12 min");
        assert_eq!(countdown(now + TimeDelta::minutes(192), now), "3 h 12 min");
        assert_eq!(countdown(at("2026-11-07T19:00:00Z"), now), "3 d 7 h");
        assert_eq!(countdown(now - TimeDelta::hours(1), now), "now");
        assert_eq!(percent(41.2), "41 %");
        assert_eq!(percent(0.5), "0.5 %");
        assert_eq!(percent(0.0), "0 %");
    }
}
