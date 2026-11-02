//! The sessions the tool sees (D-20; EP-12): every Claude Code session of the owner's projects,
//! whoever started it, from four feeds: the hooks and the status lines the window's server hears
//! (`signals`), the window's own headless runs, and a poll of `claude agents --json` ([`agents`]),
//! the documented list of live sessions, which covers what the hooks miss (a session started while
//! no window ran, an end hook that never fired). A session's folder is reconciled to the registered
//! project that holds it; its mode says who started it (a visual launch of the tool's, a headless
//! run, or the owner by hand) and its status what it does now. The model keeps no transcript and
//! writes nothing; [`record`] keeps the ledger's rows of the sessions it sees start and end.

pub mod agents;

use std::collections::HashSet;
use std::path::{Path, PathBuf};

use chrono::{DateTime, TimeDelta, Utc};

use crate::ledger::{self, Ledger, NewSession};
use crate::paths;
use crate::signals::{Hook, StatusLine};
use agents::Agent;

/// How far back a session's start looks for the visual launch it came from (EP-11's row).
pub const VISUAL_CLAIM: TimeDelta = TimeDelta::minutes(30);

/// How long an ended session stays in view.
pub const ENDED_FOR: TimeDelta = TimeDelta::minutes(10);

/// A session nothing has mentioned for this long is dropped: a poll that cannot list it and no
/// hook.
pub const SILENT_FOR: TimeDelta = TimeDelta::hours(6);

/// The polls in a row that must leave out a session they listed before it counts as ended.
const MISSED_POLLS: u8 = 2;

/// Who started a session (D-16, D-26).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    /// A visual launch of the tool's: its ledger row is claimed by the session's start.
    Visual,
    /// A `-p` run: the window's own, or another program's.
    Headless,
    /// The owner, by hand: in VS Code, a terminal or the desktop app.
    Manual,
}

impl Mode {
    pub fn as_str(self) -> &'static str {
        match self {
            Mode::Visual => "visual",
            Mode::Headless => "headless",
            Mode::Manual => "manual",
        }
    }

    fn parse(text: &str) -> Option<Mode> {
        [Mode::Visual, Mode::Headless, Mode::Manual]
            .into_iter()
            .find(|mode| mode.as_str() == text)
    }
}

/// What a session does now.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    Busy,
    /// It waits for the owner: a permission prompt, a question.
    Waiting,
    /// Its turn is over; it waits for a prompt.
    Idle,
    Done,
    Failed,
}

impl Status {
    pub fn as_str(self) -> &'static str {
        match self {
            Status::Busy => "busy",
            Status::Waiting => "waiting",
            Status::Idle => "idle",
            Status::Done => "done",
            Status::Failed => "failed",
        }
    }

    pub fn is_over(self) -> bool {
        matches!(self, Status::Done | Status::Failed)
    }
}

/// A registered project's name and folder, for the reconciliation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Registered {
    pub name: String,
    pub folder: PathBuf,
}

/// The registered project whose folder is `folder` or holds it, case aside; the deepest wins.
pub fn project_of(projects: &[Registered], folder: &Path) -> Option<String> {
    if folder.as_os_str().is_empty() {
        return None;
    }
    let key = |path: &Path| {
        paths::normalize(path)
            .to_string_lossy()
            .replace('/', "\\")
            .trim_end_matches('\\')
            .to_lowercase()
    };
    let inner = key(folder);
    projects
        .iter()
        .filter_map(|project| {
            let outer = key(&project.folder);
            let holds =
                !outer.is_empty() && (inner == outer || inner.starts_with(&format!("{outer}\\")));
            holds.then_some((outer.len(), &project.name))
        })
        .max_by_key(|(depth, _)| *depth)
        .map(|(_, name)| name.clone())
}

/// Whether a session's surface is one the owner works in: an SDK's (`sdk-…`, a `-p` run) is not.
/// A session that does not say is taken for an interactive one.
pub fn interactive(entrypoint: Option<&str>) -> bool {
    !entrypoint.is_some_and(|surface| surface.trim().to_ascii_lowercase().starts_with("sdk"))
}

/// One session the tool sees.
#[derive(Debug, Clone, PartialEq)]
pub struct Seen {
    pub session_id: String,
    /// The registered project that holds its folder; none for a folder outside the registry.
    pub project: Option<String>,
    pub folder: PathBuf,
    pub mode: Mode,
    pub status: Status,
    /// What a waiting session waits for, in words.
    pub waiting_for: Option<String>,
    /// Its name or generated title.
    pub name: Option<String>,
    /// Its surface: `claude-vscode`, `cli`, `sdk-cli`, …
    pub entrypoint: Option<String>,
    /// `interactive` or `background`, from the poll.
    pub kind: Option<String>,
    pub started: Option<DateTime<Utc>>,
    /// When a feed last mentioned it.
    pub heard: DateTime<Utc>,
    /// The latest status line's figures (D-23).
    pub line: Option<StatusLine>,
    /// The poll has listed it: its absence from later polls ends it.
    listed: bool,
    missed: u8,
}

impl Seen {
    fn new(session_id: &str, folder: &Path, project: Option<String>, now: DateTime<Utc>) -> Self {
        Self {
            session_id: session_id.to_owned(),
            project,
            folder: folder.to_owned(),
            mode: Mode::Manual,
            status: Status::Idle,
            waiting_for: None,
            name: None,
            entrypoint: None,
            kind: None,
            started: None,
            heard: now,
            line: None,
            listed: false,
            missed: 0,
        }
    }

    /// What it does, after its subject: `waits for a permission prompt`, `is busy`, `has ended`.
    pub fn doing(&self) -> String {
        match (self.status, &self.waiting_for) {
            (Status::Waiting, Some(what)) => format!("waits for {what}"),
            (Status::Waiting, None) => "waits for the owner".to_owned(),
            (Status::Busy | Status::Idle, _) => format!("is {}", self.status.as_str()),
            (Status::Done, _) => "has ended".to_owned(),
            (Status::Failed, _) => "has failed".to_owned(),
        }
    }

    /// One line on it for a project's row: `a visual session waits for a permission prompt
    /// (since 10:02)`.
    pub fn summary(&self) -> String {
        let since = self
            .started
            .map(|at| {
                format!(
                    " (since {})",
                    at.with_timezone(&chrono::Local).format("%H:%M")
                )
            })
            .unwrap_or_default();
        format!("a {} session {}{since}", self.mode.as_str(), self.doing())
    }
}

/// What a signal asks of the ledger ([`record`]).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Change {
    Started {
        session_id: String,
        project: Option<String>,
        interactive: bool,
        at: DateTime<Utc>,
    },
    Ended {
        session_id: String,
        at: DateTime<Utc>,
    },
    Nothing,
}

/// The sessions seen, and how the last poll went.
#[derive(Debug, Clone, Default)]
pub struct Sessions {
    seen: Vec<Seen>,
    /// When the poll last answered.
    pub polled: Option<DateTime<Utc>>,
    /// Why the last poll failed, while it does.
    pub poll_problem: Option<String>,
}

impl Sessions {
    /// Every session seen, the latest start first.
    pub fn all(&self) -> Vec<&Seen> {
        let mut all: Vec<&Seen> = self.seen.iter().collect();
        all.sort_by(|a, b| b.started.cmp(&a.started).then(b.heard.cmp(&a.heard)));
        all
    }

    /// The sessions of the project called `name`, the latest start first.
    pub fn of_project(&self, name: &str) -> Vec<&Seen> {
        self.all()
            .into_iter()
            .filter(|seen| {
                seen.project
                    .as_deref()
                    .is_some_and(|project| project.eq_ignore_ascii_case(name))
            })
            .collect()
    }

    /// How many sessions that have not ended run outside the registered projects.
    pub fn outside(&self) -> usize {
        self.seen
            .iter()
            .filter(|seen| seen.project.is_none() && !seen.status.is_over())
            .count()
    }

    pub fn get(&self, session_id: &str) -> Option<&Seen> {
        self.seen.iter().find(|seen| seen.session_id == session_id)
    }

    /// Records who started the session, once the ledger has said (a visual launch claimed).
    pub fn set_mode(&mut self, session_id: &str, mode: Mode) {
        if let Some(seen) = self.seen.iter_mut().find(|s| s.session_id == session_id) {
            seen.mode = mode;
        }
    }

    /// The session `session_id`, made when it is new: its folder reconciled, its mode taken
    /// from its surface.
    fn entry(
        &mut self,
        session_id: &str,
        folder: &Path,
        entrypoint: Option<&str>,
        projects: &[Registered],
        now: DateTime<Utc>,
    ) -> &mut Seen {
        let at = match self.seen.iter().position(|s| s.session_id == session_id) {
            Some(at) => at,
            None => {
                let mut seen = Seen::new(session_id, folder, project_of(projects, folder), now);
                if !interactive(entrypoint) {
                    seen.mode = Mode::Headless;
                }
                self.seen.push(seen);
                self.seen.len() - 1
            }
        };
        let seen = &mut self.seen[at];
        if seen.project.is_none() && !folder.as_os_str().is_empty() {
            seen.folder = folder.to_owned();
            seen.project = project_of(projects, folder);
        }
        if let Some(surface) = entrypoint {
            seen.entrypoint = Some(surface.to_owned());
        }
        seen.heard = now;
        seen
    }

    /// Takes in a hook (D-20): a start makes the session idle, a notification that asks for the
    /// owner makes it wait, a stop ends its turn, an end ends it.
    pub fn hook(&mut self, hook: &Hook, projects: &[Registered], now: DateTime<Utc>) -> Change {
        if hook.session_id.is_empty() {
            return Change::Nothing;
        }
        let entrypoint = hook.entrypoint.as_deref();
        let seen = self.entry(&hook.session_id, &hook.cwd, entrypoint, projects, now);
        match hook.event.as_str() {
            "SessionStart" => {
                seen.status = Status::Idle;
                seen.waiting_for = None;
                seen.started.get_or_insert(now);
                Change::Started {
                    session_id: seen.session_id.clone(),
                    project: seen.project.clone(),
                    interactive: interactive(entrypoint),
                    at: now,
                }
            }
            "Notification" => {
                match hook.notification_type.as_deref() {
                    Some("idle_prompt") => {
                        seen.status = Status::Idle;
                        seen.waiting_for = None;
                    }
                    Some(kind) => {
                        if let Some(what) = waits_for(kind) {
                            seen.status = Status::Waiting;
                            seen.waiting_for = Some(what.to_owned());
                        }
                    }
                    None => {}
                }
                Change::Nothing
            }
            "Stop" => {
                seen.status = Status::Idle;
                seen.waiting_for = None;
                Change::Nothing
            }
            "SessionEnd" => {
                seen.status = Status::Done;
                seen.waiting_for = None;
                Change::Ended {
                    session_id: seen.session_id.clone(),
                    at: now,
                }
            }
            _ => Change::Nothing,
        }
    }

    /// Takes in a status line (D-23): its figures, and a session that runs.
    pub fn status_line(&mut self, line: &StatusLine, projects: &[Registered], now: DateTime<Utc>) {
        if line.session_id.is_empty() {
            return;
        }
        let entrypoint = line.entrypoint.as_deref();
        let seen = self.entry(&line.session_id, line.folder(), entrypoint, projects, now);
        if seen.status.is_over() {
            seen.status = Status::Idle;
        }
        if let Some(name) = &line.session_name {
            seen.name = Some(name.clone());
        }
        seen.line = Some(line.clone());
    }

    /// Takes in a poll's listing: each listed session's status as it stands, and the end of a
    /// session it listed before and has left out [`MISSED_POLLS`] times since.
    pub fn poll(&mut self, agents: &[Agent], projects: &[Registered], now: DateTime<Utc>) {
        self.polled = Some(now);
        self.poll_problem = None;
        let mut listed = HashSet::new();
        for agent in agents {
            let Some(id) = agent.session_id.as_deref().filter(|id| !id.is_empty()) else {
                continue;
            };
            listed.insert(id.to_owned());
            let seen = self.entry(id, agent.folder(), None, projects, now);
            seen.listed = true;
            seen.missed = 0;
            if let Some(name) = &agent.name {
                seen.name = Some(name.clone());
            }
            if agent.kind.is_some() {
                seen.kind = agent.kind.clone();
            }
            if seen.started.is_none() {
                seen.started = agent.started();
            }
            if let Some((status, waiting_for)) = agent.status_now() {
                seen.status = status;
                seen.waiting_for = waiting_for;
            }
        }
        for seen in &mut self.seen {
            if seen.listed && !seen.status.is_over() && !listed.contains(&seen.session_id) {
                seen.missed += 1;
                if seen.missed >= MISSED_POLLS {
                    seen.status = Status::Done;
                    seen.waiting_for = None;
                    seen.heard = now;
                }
            }
        }
    }

    /// A poll that failed: said, and nothing ended for it.
    pub fn poll_failed(&mut self, why: String) {
        self.poll_problem = Some(why);
    }

    /// Lets ended sessions go after [`ENDED_FOR`] and silent ones after [`SILENT_FOR`].
    pub fn expire(&mut self, now: DateTime<Utc>) {
        self.seen.retain(|seen| {
            let quiet = now - seen.heard;
            if seen.status.is_over() {
                quiet < ENDED_FOR
            } else {
                quiet < SILENT_FOR
            }
        });
    }

    /// A session to show as it stands (a screenshot run's canned sessions).
    pub fn insert(&mut self, seen: Seen) {
        self.seen.retain(|s| s.session_id != seen.session_id);
        self.seen.push(seen);
    }
}

impl Seen {
    /// A session as a screenshot run shows it.
    pub fn canned(
        session_id: &str,
        project: Option<&str>,
        mode: Mode,
        status: Status,
        waiting_for: Option<&str>,
        started: DateTime<Utc>,
    ) -> Self {
        Self {
            mode,
            status,
            waiting_for: waiting_for.map(str::to_owned),
            started: Some(started),
            ..Seen::new(
                session_id,
                Path::new(""),
                project.map(str::to_owned),
                started,
            )
        }
    }
}

/// What a notification that asks for the owner waits for, in words; `None` for one that does not.
fn waits_for(kind: &str) -> Option<&'static str> {
    match kind {
        "permission_prompt" => Some("a permission prompt"),
        "elicitation_dialog" | "elicitation_url_dialog" => Some("an MCP server's question"),
        "agent_needs_input" => Some("input"),
        _ => None,
    }
}

/// Keeps the ledger's rows of the sessions the hooks see (D-26; EP-12): a start claims the newest
/// open visual launch of its project from the last [`VISUAL_CLAIM`] (EP-11's row gains its session
/// id), else opens a manual row for an interactive session of a registered project; an end closes
/// a visual or manual row. A session the ledger knows already (the runner's, a resumed one) keeps
/// its row. Returns the mode the ledger gives the session, when it gives one.
pub fn record(ledger: &Ledger, change: &Change) -> Result<Option<Mode>, ledger::Error> {
    match change {
        Change::Started {
            session_id,
            project: Some(project),
            interactive: true,
            at,
        } => {
            if let Some(mode) = ledger.mode_of(session_id)? {
                return Ok(Mode::parse(&mode));
            }
            if ledger
                .claim_visual(project, session_id, *at - VISUAL_CLAIM)?
                .is_some()
            {
                return Ok(Some(Mode::Visual));
            }
            ledger.start_session(&NewSession {
                session_id: Some(session_id.as_str()),
                project: project.as_str(),
                ep: None,
                mode: Mode::Manual.as_str(),
                model: None,
                effort: None,
                ultracode: false,
                started_at: *at,
            })?;
            Ok(Some(Mode::Manual))
        }
        Change::Ended { session_id, at } => {
            ledger.end_seen(session_id, *at)?;
            Ok(None)
        }
        _ => Ok(None),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::files::tests::Scratch;

    fn projects() -> Vec<Registered> {
        ["alpha", "alpha-tools", "beta"]
            .into_iter()
            .map(|name| Registered {
                name: name.to_owned(),
                folder: PathBuf::from(format!(r"C:\Data\{name}")),
            })
            .chain([Registered {
                name: "nested".to_owned(),
                folder: PathBuf::from(r"C:\Data\beta\nested"),
            }])
            .collect()
    }

    fn at(minute: i64) -> DateTime<Utc> {
        DateTime::parse_from_rfc3339("2026-11-02T10:00:00Z")
            .unwrap()
            .with_timezone(&Utc)
            + TimeDelta::minutes(minute)
    }

    fn hook(event: &str, id: &str, cwd: &str) -> Hook {
        Hook {
            event: event.to_owned(),
            session_id: id.to_owned(),
            cwd: PathBuf::from(cwd),
            entrypoint: Some("claude-vscode".to_owned()),
            ..Hook::default()
        }
    }

    #[test]
    fn a_folder_belongs_to_the_project_that_holds_it() {
        let projects = projects();
        let of = |folder: &str| project_of(&projects, Path::new(folder));
        assert_eq!(of(r"C:\Data\alpha").as_deref(), Some("alpha"));
        assert_eq!(of(r"c:\data\ALPHA\crates\core").as_deref(), Some("alpha"));
        assert_eq!(of("C:/Data/alpha/").as_deref(), Some("alpha"));
        // A name that starts like another is not inside it; the deepest folder wins.
        assert_eq!(of(r"C:\Data\alpha-tools\x").as_deref(), Some("alpha-tools"));
        assert_eq!(of(r"C:\Data\beta\nested\deep").as_deref(), Some("nested"));
        assert_eq!(of(r"C:\Data\TEMP\scratch"), None);
        assert_eq!(of(""), None);
        assert!(interactive(Some("claude-vscode")));
        assert!(interactive(Some("cli")));
        assert!(interactive(None));
        assert!(!interactive(Some("sdk-cli")));
    }

    #[test]
    fn hooks_move_a_session_through_its_life() {
        let projects = projects();
        let mut sessions = Sessions::default();
        let change = sessions.hook(
            &hook("SessionStart", "s1", r"C:\Data\alpha"),
            &projects,
            at(0),
        );
        assert_eq!(
            change,
            Change::Started {
                session_id: "s1".to_owned(),
                project: Some("alpha".to_owned()),
                interactive: true,
                at: at(0)
            }
        );
        let seen = sessions.get("s1").unwrap();
        assert_eq!((seen.mode, seen.status), (Mode::Manual, Status::Idle));
        assert_eq!(seen.started, Some(at(0)));
        let mut note = hook("Notification", "s1", r"C:\Data\alpha");
        note.notification_type = Some("permission_prompt".to_owned());
        sessions.hook(&note, &projects, at(1));
        assert_eq!(
            sessions.get("s1").unwrap().doing(),
            "waits for a permission prompt"
        );
        // An authentication notice changes nothing; an idle prompt is idle.
        note.notification_type = Some("auth_success".to_owned());
        sessions.hook(&note, &projects, at(2));
        assert_eq!(sessions.get("s1").unwrap().status, Status::Waiting);
        sessions.hook(&hook("Stop", "s1", r"C:\Data\alpha"), &projects, at(3));
        assert_eq!(sessions.get("s1").unwrap().doing(), "is idle");
        sessions.set_mode("s1", Mode::Visual);
        assert!(
            sessions.of_project("ALPHA")[0]
                .summary()
                .starts_with("a visual session is idle (since ")
        );
        let end = sessions.hook(
            &hook("SessionEnd", "s1", r"C:\Data\alpha"),
            &projects,
            at(4),
        );
        assert_eq!(
            end,
            Change::Ended {
                session_id: "s1".to_owned(),
                at: at(4)
            }
        );
        assert_eq!(sessions.get("s1").unwrap().status, Status::Done);
        // Ended sessions leave after ten minutes.
        sessions.expire(at(13));
        assert!(sessions.get("s1").is_some());
        sessions.expire(at(15));
        assert!(sessions.get("s1").is_none());
        // Another program's `-p` run is headless; a session outside the registry is counted.
        let mut run = hook("SessionStart", "s2", r"C:\Data\beta\work");
        run.entrypoint = Some("sdk-cli".to_owned());
        let change = sessions.hook(&run, &projects, at(20));
        assert!(matches!(
            change,
            Change::Started {
                interactive: false,
                ..
            }
        ));
        assert_eq!(sessions.get("s2").unwrap().mode, Mode::Headless);
        sessions.hook(
            &hook("SessionStart", "s3", r"C:\Elsewhere"),
            &projects,
            at(20),
        );
        assert_eq!(sessions.outside(), 1);
        assert_eq!(sessions.of_project("beta").len(), 1);
        // A hook with no session id is passed over.
        assert_eq!(
            sessions.hook(&hook("Stop", "", r"C:\Data\beta"), &projects, at(21)),
            Change::Nothing
        );
    }

    #[test]
    fn the_poll_reconciles_and_ends_what_it_stops_listing() {
        let projects = projects();
        let mut sessions = Sessions::default();
        let listing = agents::parse(
            r#"[{"pid": 1, "cwd": "C:\\Data\\beta", "kind": "interactive", "startedAt": 1793613600000,
                 "sessionId": "p1", "name": "beta-x", "status": "busy"},
                {"pid": 2, "cwd": "C:\\Data\\alpha", "kind": "interactive", "sessionId": "p2", "status": "waiting",
                 "waitingFor": "permission prompt"}]"#,
        )
        .unwrap();
        sessions.poll(&listing, &projects, at(0));
        let p1 = sessions.get("p1").unwrap();
        assert_eq!(
            (p1.project.as_deref(), p1.status),
            (Some("beta"), Status::Busy)
        );
        assert_eq!(p1.name.as_deref(), Some("beta-x"));
        assert_eq!(p1.kind.as_deref(), Some("interactive"));
        assert_eq!(
            sessions.get("p2").unwrap().doing(),
            "waits for permission prompt"
        );
        // A status line names the session and keeps its figures.
        let line = StatusLine {
            session_id: "p2".to_owned(),
            session_name: Some("alpha EP-3".to_owned()),
            ..StatusLine::default()
        };
        sessions.status_line(&line, &projects, at(0));
        assert_eq!(
            sessions.get("p2").unwrap().name.as_deref(),
            Some("alpha EP-3")
        );
        assert!(sessions.get("p2").unwrap().line.is_some());
        // p1 leaves the listing: one poll is not enough, two are.
        sessions.poll(&listing[1..], &projects, at(1));
        assert_eq!(sessions.get("p1").unwrap().status, Status::Busy);
        sessions.poll(&listing[1..], &projects, at(2));
        assert_eq!(sessions.get("p1").unwrap().status, Status::Done);
        // A session the poll never listed (a `-p` run) is not ended by its absence.
        let mut run = hook("SessionStart", "h1", r"C:\Data\alpha");
        run.entrypoint = Some("sdk-cli".to_owned());
        sessions.hook(&run, &projects, at(2));
        sessions.poll(&listing[1..], &projects, at(3));
        sessions.poll(&listing[1..], &projects, at(4));
        assert_eq!(sessions.get("h1").unwrap().status, Status::Idle);
        // Silent for six hours, it goes.
        sessions.expire(at(2) + SILENT_FOR);
        assert!(sessions.get("h1").is_none());
        sessions.poll_failed("claude agents --json: did not start".to_owned());
        assert!(sessions.poll_problem.is_some());
    }

    #[test]
    fn the_ledger_keeps_the_rows_of_what_the_hooks_see() {
        let scratch = Scratch::new("sessions-record");
        let ledger = Ledger::open(&scratch.0.join("ledger.sqlite")).unwrap();
        let projects = projects();
        let mut sessions = Sessions::default();
        // A visual launch of alpha, recorded when its link went to VS Code (EP-11).
        ledger
            .start_session(&NewSession {
                session_id: None,
                project: "alpha",
                ep: Some("EP-5"),
                mode: "visual",
                model: None,
                effort: None,
                ultracode: false,
                started_at: at(0),
            })
            .unwrap();
        let start = sessions.hook(
            &hook("SessionStart", "v1", r"C:\Data\alpha"),
            &projects,
            at(1),
        );
        assert_eq!(record(&ledger, &start).unwrap(), Some(Mode::Visual));
        // A second start in alpha finds no launch left: a manual row.
        let other = sessions.hook(
            &hook("SessionStart", "m1", r"C:\Data\alpha"),
            &projects,
            at(2),
        );
        assert_eq!(record(&ledger, &other).unwrap(), Some(Mode::Manual));
        // A resume of a known session keeps its row.
        assert_eq!(record(&ledger, &other).unwrap(), Some(Mode::Manual));
        // Outside the registry, or a `-p` run: no row.
        let outside = sessions.hook(
            &hook("SessionStart", "o1", r"C:\Elsewhere"),
            &projects,
            at(2),
        );
        assert_eq!(record(&ledger, &outside).unwrap(), None);
        let mut run = hook("SessionStart", "h1", r"C:\Data\beta");
        run.entrypoint = Some("sdk-cli".to_owned());
        let run = sessions.hook(&run, &projects, at(2));
        assert_eq!(record(&ledger, &run).unwrap(), None);
        let end = sessions.hook(
            &hook("SessionEnd", "v1", r"C:\Data\alpha"),
            &projects,
            at(30),
        );
        record(&ledger, &end).unwrap();
        let rows = ledger.sessions().unwrap();
        assert_eq!(rows.len(), 2);
        let visual = &rows[0];
        assert_eq!(
            (visual.session_id.as_deref(), visual.ep.as_deref()),
            (Some("v1"), Some("EP-5"))
        );
        assert_eq!(visual.ended_at.as_deref(), Some("2026-11-02T10:30:00Z"));
        assert_eq!(
            (rows[1].mode.as_str(), rows[1].ended_at.as_deref()),
            ("manual", None)
        );
    }
}
