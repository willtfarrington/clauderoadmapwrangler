//! The phone's pages (D-14, D-28; EP-30): askama templates in the crate's `templates` folder,
//! rendered from what the window publishes (`published`), and the stylesheet they share, served
//! from the exe. A page loads nothing from outside, works with no script, and follows the phone's
//! own light or dark setting. EP-30's page is the project list, which reads only; EP-30a adds a
//! project's page (its next brief's note, what waits, the inbox, a message to it) and the
//! decisions, whose forms post to the window and come back with one line saying what happened;
//! EP-30b adds the actions (a project's start page, a run's Stop, the autopilot's Pause and
//! Resume), the list's usage strip and autopilot block, and the live updates: every page names
//! the paths of the parts it shows (`data-listen`) and marks each part (`data-part`) for the
//! tool's own script, which fetches the page again when the event stream names one of them.

use askama::Template;
use chrono::{DateTime, Local, Utc};
use crw_core::badges::short_age;
use crw_core::export::escape;
use crw_core::registry::State;
use crw_core::usage;

use crw_core::doctor::{Group, Level};

use crate::form::segment;
use crate::markup::{self, Block};
use crate::published::{self, Asking, How, Item, Line, Next, Published, SessionPage};
use crate::server::Said;
use crate::transcript::{self, Entry};

/// The stylesheet every page links, at `/m/app.css`.
pub const STYLESHEET: &str = include_str!("../templates/app.css");

/// The script every page loads, at `/m/app.js` (EP-30b): the live updates. A page works the same
/// without it.
pub const SCRIPT: &str = include_str!("../templates/app.js");

/// A run of a title's inline markup as the page draws it: its text and the element around it
/// (`code`, `strong`, `s`, or none).
pub struct Run {
    pub text: String,
    pub tag: &'static str,
}

/// `text`'s inline markup (`markup::spans`, as the row draws its title): code first, then bold,
/// then struck; a link's text is plain.
fn runs(text: &str) -> Vec<Run> {
    crate::markup::spans(text)
        .into_iter()
        .map(|(text, marks)| Run {
            text,
            tag: if marks.code {
                "code"
            } else if marks.bold {
                "strong"
            } else if marks.struck {
                "s"
            } else {
                ""
            },
        })
        .collect()
}

/// A group as the page shows it: folded for finished work, as the window folds it.
struct GroupView {
    heading: &'static str,
    open: bool,
    projects: Vec<LineView>,
}

/// A project as the page shows it, its words made beforehand so the template stays plain.
struct LineView {
    name: String,
    /// Its page's path (EP-30a).
    link: String,
    state: &'static str,
    /// The marks after the state, each with its class.
    marks: Vec<(String, &'static str)>,
    next_id: Option<String>,
    size: Option<String>,
    title: Vec<Run>,
    no_next: Option<String>,
    missing: bool,
    done: usize,
    total: usize,
    progress: String,
    /// What waits on the owner, each with its class.
    waits: Vec<(String, &'static str)>,
    sessions: Vec<String>,
    badges: Vec<crate::published::Badge>,
}

/// The marks after a project's state, each with its class: pinned, a state change proposed, the
/// autopilot on.
fn marks(pinned: bool, proposes: bool, autopilot: bool) -> Vec<(String, &'static str)> {
    let mut marks = Vec::new();
    if pinned {
        marks.push(("pinned".to_owned(), "weak"));
    }
    if proposes {
        marks.push(("state change proposed".to_owned(), "warn"));
    }
    if autopilot {
        marks.push(("autopilot on".to_owned(), "good"));
    }
    marks
}

impl LineView {
    fn of(line: &Line) -> Self {
        let marks = marks(line.pinned, line.proposes, line.autopilot);
        let (next_id, size, title, no_next, missing) = match &line.next {
            Next::Brief { id, size, title } => {
                (Some(id.clone()), size.clone(), runs(title), None, false)
            }
            Next::None { words, missing } => {
                (None, None, Vec::new(), Some(words.clone()), *missing)
            }
        };
        let counts = line.counts;
        let progress = if counts.total == 0 {
            String::new()
        } else {
            let stretch = match counts.stretch {
                0 => String::new(),
                n => format!(", {n} stretch"),
            };
            format!(
                "{} of {} rows done ({:.0} %){stretch}",
                counts.done,
                counts.total,
                counts.done as f64 * 100.0 / counts.total as f64
            )
        };
        let mut waits = Vec::new();
        match line.owner_items {
            0 => {}
            1 => waits.push(("1 owner task".to_owned(), "warn")),
            n => waits.push((format!("{n} owner tasks"), "warn")),
        }
        if line.messages > 0 {
            let messages = match line.messages {
                1 => "1 inbox message".to_owned(),
                n => format!("{n} inbox messages"),
            };
            match line.urgent {
                0 => waits.push((messages, "warn")),
                n => waits.push((format!("{messages}, {n} urgent"), "bad")),
            }
        }
        Self {
            name: line.name.clone(),
            link: project_link(&line.name),
            state: line.state.as_str(),
            marks,
            next_id,
            size,
            title,
            no_next,
            missing,
            done: counts.done,
            total: counts.total,
            progress,
            waits,
            sessions: line.sessions.clone(),
            badges: line.badges.clone(),
        }
    }
}

#[derive(Template)]
#[template(path = "list.html")]
struct ListPage {
    as_of: String,
    /// The snapshot's version the page is drawn from, for the event stream's `since` (EP-30b).
    version: u64,
    /// The outcome of the Pause or Resume that led here (EP-30b).
    said: Option<SaidView>,
    summary: String,
    /// What waits for an answer, for the link to the decisions page (EP-30a).
    decisions: String,
    problem: Option<String>,
    reading: bool,
    groups: Vec<GroupView>,
    unadopted: Vec<String>,
    /// The usage strip (EP-30b): none while the meter is off.
    usage: Option<UsageView>,
    autopilot: AutopilotView,
}

/// The usage strip as the page draws it (EP-30b), its ages drawn now.
struct UsageView {
    figures: Vec<FigureView>,
    session_closed: bool,
    degraded: Option<String>,
}

/// One figure: its name, its share and its class, its reset and its reading in words.
struct FigureView {
    name: &'static str,
    percent: String,
    class: &'static str,
    reset: String,
    read: String,
}

impl UsageView {
    fn of(usage: &published::Usage, now: DateTime<Utc>) -> Option<Self> {
        usage.on.then(|| Self {
            figures: usage
                .figures
                .iter()
                .map(|figure| FigureView {
                    name: figure.name,
                    percent: usage::percent(figure.percent),
                    class: if figure.percent >= usage::ALARM {
                        "bad"
                    } else if figure.percent >= usage::WARN {
                        "warn"
                    } else {
                        "good"
                    },
                    reset: figure.resets_at.map_or_else(
                        || "no reset time given".to_owned(),
                        |at| {
                            format!(
                                "resets in {}, {}",
                                usage::countdown(at, now),
                                at.with_timezone(&Local).format("%a %H:%M")
                            )
                        },
                    ),
                    read: format!("read {} from {}", ago(now, figure.at), figure.from),
                })
                .collect(),
            session_closed: usage.session_closed,
            degraded: usage.degraded.clone(),
        })
    }
}

/// `just now`, `3 min ago`, as the meter says a reading's age.
fn ago(now: DateTime<Utc>, at: DateTime<Utc>) -> String {
    match short_age(now - at).as_str() {
        "now" => "just now".to_owned(),
        age => format!("{age} ago"),
    }
}

/// The autopilot block as the page draws it (EP-30b), in the Autopilot pane's words (EP-52).
struct AutopilotView {
    state: &'static str,
    class: &'static str,
    projects: String,
    /// The day's chained runs against the total in all projects, in the section's words, and
    /// its classes, `warn` once the total is reached (EP-31b); read-only (D-28).
    today: String,
    today_class: &'static str,
    /// Paused by the owner: the form resumes; else it pauses.
    paused: bool,
    held_by_usage: Vec<String>,
    held: Vec<(String, String)>,
    plans: Vec<(String, String, String)>,
    /// Each line's clock time, project and words.
    log: Vec<(String, String, String)>,
    problem: Option<String>,
}

impl AutopilotView {
    fn of(autopilot: &published::Autopilot) -> Self {
        let (state, class) = if autopilot.paused {
            ("paused by you", "warn")
        } else if !autopilot.held_by_usage.is_empty() {
            ("held by usage", "warn")
        } else if autopilot.on.is_empty() {
            ("off everywhere", "weak")
        } else {
            ("on", "good")
        };
        let projects = match autopilot.on.as_slice() {
            [] => "no project has it on".to_owned(),
            [one] => format!("on in {one}"),
            all => format!("on in {} projects: {}", all.len(), all.join(", ")),
        };
        let (today, reached) =
            crate::autopilot::day_line(autopilot.chained_today, autopilot.daily_total);
        Self {
            state,
            class,
            projects,
            today,
            today_class: if reached { "small warn" } else { "small" },
            paused: autopilot.paused,
            held_by_usage: autopilot.held_by_usage.clone(),
            held: autopilot.held.clone(),
            plans: autopilot.plans.clone(),
            log: autopilot
                .log
                .iter()
                .map(|line| {
                    (
                        line.at.map(local_time).unwrap_or_default(),
                        line.project.clone().unwrap_or_default(),
                        line.words.clone(),
                    )
                })
                .collect(),
            problem: autopilot.problem.clone(),
        }
    }
}

/// The line a form's redirect brings to its page (EP-30a): its words and whether it was done.
struct SaidView {
    done: bool,
    words: String,
}

impl SaidView {
    fn of(said: Option<&Said>) -> Option<Self> {
        said.map(|said| match said {
            Ok(words) => Self {
                done: true,
                words: words.clone(),
            },
            Err(words) => Self {
                done: false,
                words: words.clone(),
            },
        })
    }
}

/// `text` set in blocks as the Detail pane sets it (`markup::blocks`), as HTML made beforehand
/// (EP-30a): a paragraph, a segment's label on a line of its own, a numbered part with its
/// number, a sentence of a long text with its bullet, a gap; every run escaped. No Markdown
/// renderer joins the tree: this is the same block model the window draws.
fn note_html(text: &str, split: bool) -> String {
    let mut html = String::new();
    for block in markup::blocks(text, split) {
        let line = match block {
            Block::Text(text) => format!("<p>{}</p>", inline(&text)),
            Block::Label(text) => format!("<p class=\"label\">{}</p>", inline(&text)),
            Block::Part(number, text) => format!(
                "<p class=\"part\"><span class=\"n\">{}</span> {}</p>",
                escape(&number),
                inline(&text)
            ),
            Block::Step(text) => format!("<p class=\"step\">{}</p>", inline(&text)),
            Block::Gap => "<div class=\"gap\"></div>".to_owned(),
        };
        html.push_str(&line);
        html.push('\n');
    }
    html
}

/// `text`'s inline markup as HTML: every run escaped (`export::escape`), code, bold and struck
/// runs in their elements, a link as its text.
fn inline(text: &str) -> String {
    let mut html = String::new();
    for run in runs(text) {
        let text = escape(&run.text);
        if run.tag.is_empty() {
            html.push_str(&text);
        } else {
            html.push_str(&format!("<{0}>{text}</{0}>", run.tag));
        }
    }
    html
}

/// The path of the project called `name`'s page.
fn project_link(name: &str) -> String {
    format!("{}{}", crate::server::PROJECT_PREFIX, segment(name))
}

/// An item waiting on the owner, as the project page shows it.
struct ItemView {
    /// The id said beside it: none when its text opens with it.
    id: String,
    /// The id its form names, said or not.
    form_id: String,
    gate: String,
    /// Its text as HTML (`note_html`).
    html: String,
    meta: String,
    tickable: bool,
}

impl ItemView {
    fn of(item: &Item) -> Self {
        // An id the text opens with (a register's `**R-2**`) is not said twice.
        let id = item
            .id
            .clone()
            .filter(|id| !item.text.contains(id.as_str()))
            .unwrap_or_default();
        Self {
            id,
            form_id: item.id.clone().unwrap_or_default(),
            gate: item.gate.clone().unwrap_or_default(),
            html: note_html(&item.text, true),
            meta: item.meta.clone(),
            tickable: item.tickable && item.id.is_some(),
        }
    }
}

/// An open message, as the project page shows it.
struct MessageView {
    id: String,
    from: String,
    urgent: bool,
    html: String,
}

/// One of the window's runs on the project's page (EP-30b), with its Stop.
struct RunView {
    key: u64,
    ep: String,
    /// Its session id, which the Stop names too (EP-31's review).
    session: String,
    state: String,
    started: String,
    stoppable: bool,
    chained: bool,
}

#[derive(Template)]
#[template(path = "project.html")]
struct ProjectPage {
    as_of: String,
    version: u64,
    said: Option<SaidView>,
    name: String,
    link: String,
    /// The start page's path, while the next brief may be started from here (EP-30b).
    start: Option<String>,
    runs: Vec<RunView>,
    state: &'static str,
    /// The list's marks after the state (pinned, a proposal, the autopilot), each with its class.
    marks: Vec<(String, &'static str)>,
    /// The page offers its forms: the folder is there and no revert is open there.
    folder: bool,
    /// A revert is open or running there, which the page says in place of its forms.
    revert: bool,
    next_id: Option<String>,
    size: Option<String>,
    title: Vec<Run>,
    no_next: Option<String>,
    /// The next brief's note as HTML (`note_html`); empty when it has none.
    note: String,
    waiting: Vec<ItemView>,
    resolved: usize,
    /// The project keeps `owner-tasks.md` (EP-50).
    keeps_tasks: bool,
    uncounted: Vec<ItemView>,
    has_inbox: bool,
    inbox_counts: String,
    messages: Vec<MessageView>,
    busy: Option<String>,
    sessions: Vec<String>,
}

/// A question's option on the decisions page.
struct OptionView {
    index: usize,
    label: String,
    description: String,
}

/// A question on the decisions page.
struct QuestionView {
    index: usize,
    header: String,
    text: String,
    several: bool,
    options: Vec<OptionView>,
}

/// A request on the decisions page, its form's fields made beforehand.
struct AskView {
    key: u64,
    id: String,
    title: String,
    tool: String,
    reason: String,
    wait: String,
    summary: String,
    input: String,
    questions: Vec<QuestionView>,
    /// Its answer resumes a deferred run.
    resumes: bool,
    /// The page the answer goes back to when it is not the decisions page: a session's (EP-31).
    back: String,
}

impl AskView {
    /// `asking`'s form on a session's page, its answer coming back there (EP-31).
    fn back_to(asking: &Asking, path: &str) -> Self {
        Self {
            back: path.to_owned(),
            ..Self::of(asking)
        }
    }

    fn of(asking: &Asking) -> Self {
        let request = &asking.request;
        let at = local_time(asking.at);
        let wait = match asking.how {
            How::Live { limit: Some(limit) } => {
                let due = asking.at + chrono::TimeDelta::minutes(limit as i64);
                format!(
                    "waiting since {at} · deferred at {} unless answered",
                    local_time(due)
                )
            }
            How::Live { limit: None } => format!("waiting since {at}"),
            How::Deferred { hook: true } => {
                format!("deferred at {at} by the defer route's hook")
            }
            How::Deferred { hook: false } => {
                format!("deferred at {at} at the project's time limit")
            }
        };
        Self {
            key: asking.key,
            id: request.id.clone(),
            title: asking.title.clone(),
            tool: request.tool.clone(),
            reason: request.reason.clone().unwrap_or_default(),
            wait,
            summary: request.summary.clone(),
            input: request.input.clone(),
            questions: request
                .questions
                .iter()
                .enumerate()
                .map(|(index, question)| QuestionView {
                    index,
                    header: question.header.clone(),
                    text: question.text.clone(),
                    several: question.multi_select,
                    options: question
                        .options
                        .iter()
                        .enumerate()
                        .map(|(index, choice)| OptionView {
                            index,
                            label: choice.label.clone(),
                            description: choice.description.clone().unwrap_or_default(),
                        })
                        .collect(),
                })
                .collect(),
            resumes: matches!(asking.how, How::Deferred { .. }),
            back: String::new(),
        }
    }
}

/// A session waiting outside the window, on the decisions page.
struct ElsewhereView {
    project: String,
    mode: &'static str,
    words: String,
    name: String,
}

#[derive(Template)]
#[template(path = "decisions.html")]
struct DecisionsPage {
    as_of: String,
    version: u64,
    said: Option<SaidView>,
    waiting: Vec<AskView>,
    deferred: Vec<AskView>,
    elsewhere: Vec<ElsewhereView>,
}

#[derive(Template)]
#[template(path = "message.html")]
struct MessagePage<'a> {
    heading: &'a str,
    words: &'a str,
}

/// The start page (EP-30b): what a start of the next brief would run, and its button or what
/// refuses it.
#[derive(Template)]
#[template(path = "start.html")]
struct StartPage {
    as_of: String,
    /// The snapshot's version when the preview was drawn, for the event stream's `since`.
    version: u64,
    name: String,
    /// The project's page, which the page listens to and links back to.
    link: String,
    next_id: Option<String>,
    size: Option<String>,
    title: Vec<Run>,
    no_next: Option<String>,
    model: (String, String),
    effort: (String, String),
    ultracode: bool,
    permission_mode: (String, String),
    downshift: Option<String>,
    run: Vec<String>,
    notes: Vec<String>,
    refusals: Vec<String>,
}

/// When the snapshot was built, in the window's local time; empty before the window's first.
fn as_of(at: DateTime<Utc>) -> String {
    if at == DateTime::<Utc>::MIN_UTC {
        String::new()
    } else {
        format!("as of {}", at.with_timezone(&Local).format("%H:%M:%S"))
    }
}

/// A clock time in the window's local time: `14:05`, or with its day when not today.
fn local_time(at: DateTime<Utc>) -> String {
    crw_core::sessions::clock(at, Local::now())
}

/// The page of the project called `name` in `published`, with the outcome of the form that led
/// here; none when the window lists no such project (EP-30a).
pub fn project(
    published: &Published,
    name: &str,
    said: Option<&Said>,
) -> Option<Result<String, askama::Error>> {
    let project = published.project(name)?;
    let (next_id, size, title, no_next) = match &project.next {
        Next::Brief { id, size, title } => (Some(id.clone()), size.clone(), runs(title), None),
        Next::None { words, .. } => (None, None, Vec::new(), Some(words.clone())),
    };
    let (has_inbox, inbox_counts, messages) = match &project.inbox {
        Some(inbox) => (
            true,
            format!("{} open of {}", inbox.open.len(), inbox.total),
            inbox
                .open
                .iter()
                .map(|message| MessageView {
                    id: message.id.clone(),
                    from: message.from.clone(),
                    urgent: message.urgent,
                    html: note_html(&message.text, true),
                })
                .collect(),
        ),
        None => (false, String::new(), Vec::new()),
    };
    // The marks the list's line shows after the state, from the project's own part (EP-30b).
    let marks = marks(project.pinned, project.proposes, project.autopilot);
    let link = project_link(&project.name);
    let start =
        (next_id.is_some() && project.folder && !project.revert).then(|| format!("{link}/start"));
    let runs = project
        .runs
        .iter()
        .map(|run| RunView {
            key: run.key,
            ep: run.ep.clone(),
            session: run.session.clone(),
            state: run.state.clone(),
            started: local_time(run.started),
            stoppable: run.stoppable,
            chained: run.chained,
        })
        .collect();
    Some(
        ProjectPage {
            as_of: as_of(published.at),
            version: published.version,
            said: SaidView::of(said),
            name: project.name.clone(),
            link,
            start,
            runs,
            state: project.state.as_str(),
            marks,
            folder: project.folder && !project.revert,
            revert: project.revert,
            next_id,
            size,
            title,
            no_next,
            note: project
                .note
                .as_deref()
                .map(|note| note_html(note, false))
                .unwrap_or_default(),
            waiting: project.waiting.iter().map(ItemView::of).collect(),
            resolved: project.resolved,
            keeps_tasks: project.keeps_tasks,
            uncounted: project.uncounted.iter().map(ItemView::of).collect(),
            has_inbox,
            inbox_counts,
            messages,
            busy: project.busy.clone(),
            sessions: project.sessions.clone(),
        }
        .render(),
    )
}

/// The decisions page of `published`, with the outcome of the form that led here (EP-30a).
pub fn decisions(published: &Published, said: Option<&Said>) -> Result<String, askama::Error> {
    let decisions = &published.decisions;
    DecisionsPage {
        as_of: as_of(published.at),
        version: published.version,
        said: SaidView::of(said),
        waiting: decisions.waiting.iter().map(AskView::of).collect(),
        deferred: decisions.deferred.iter().map(AskView::of).collect(),
        elsewhere: decisions
            .elsewhere
            .iter()
            .map(|seen| ElsewhereView {
                project: seen.project.clone(),
                mode: seen.mode,
                words: seen.words.clone(),
                name: seen.name.clone().unwrap_or_default(),
            })
            .collect(),
    }
    .render()
}

/// The start page of `start`, the window's preview, drawn at `now` when the snapshot stood at
/// `version` (EP-30b).
pub fn start(
    start: &published::Start,
    now: DateTime<Utc>,
    version: u64,
) -> Result<String, askama::Error> {
    let (next_id, size, title, no_next) = match &start.next {
        Next::Brief { id, size, title } => (Some(id.clone()), size.clone(), runs(title), None),
        Next::None { words, .. } => (None, None, Vec::new(), Some(words.clone())),
    };
    StartPage {
        as_of: as_of(now),
        version,
        name: start.project.clone(),
        link: project_link(&start.project),
        next_id,
        size,
        title,
        no_next,
        model: start.model.clone(),
        effort: start.effort.clone(),
        ultracode: start.ultracode,
        permission_mode: start.permission_mode.clone(),
        downshift: start.downshift.clone(),
        run: start.run.clone(),
        notes: start.notes.clone(),
        refusals: start.refusals.clone(),
    }
    .render()
}

/// One entry of a session's transcript as its page draws it (EP-31): its place in the run's
/// transcript, which the script's append mode keys on, and its HTML made beforehand, every run
/// escaped.
struct EntryView {
    n: usize,
    html: String,
}

/// A window session's page (EP-31).
#[derive(Template)]
#[template(path = "session.html")]
struct SessionView {
    as_of: String,
    version: u64,
    said: Option<SaidView>,
    /// The page's own path, which it listens to and its forms come back to.
    path: String,
    /// The run's session id, which its Stop names (EP-31's review).
    id: String,
    title: String,
    project: String,
    project_link: String,
    state: String,
    state_class: &'static str,
    facts: String,
    autopilot: Option<String>,
    dropped: Vec<String>,
    ended: Option<String>,
    /// The run's number and brief, which its Stop names, while it may be stopped.
    stop: Option<(u64, String)>,
    earlier: String,
    entries: Vec<EntryView>,
    streaming: String,
    waiting: Vec<AskView>,
    deferred: Vec<AskView>,
}

/// The sessions page (EP-31).
#[derive(Template)]
#[template(path = "sessions.html")]
struct SessionsView {
    as_of: String,
    version: u64,
    starting: bool,
    window: Vec<WindowView>,
    elsewhere: Vec<SeenView>,
    outside: String,
    poll_problem: Option<String>,
    rows: Vec<RowView>,
    problem: Option<String>,
}

/// One of the window's sessions on the sessions page.
struct WindowView {
    link: String,
    title: String,
    state: String,
    class: &'static str,
    facts: String,
    last: String,
    chained: bool,
}

/// A session started elsewhere on the sessions page.
struct SeenView {
    project: String,
    mode: String,
    status: &'static str,
    class: &'static str,
    waiting: bool,
    doing: String,
    since: String,
    name: String,
    figures: String,
}

/// A ledger row on the sessions page.
struct RowView {
    when: String,
    title: String,
    link: Option<String>,
    mode: String,
    state: String,
    class: &'static str,
    facts: String,
    result: Option<String>,
}

/// The notifications page (EP-31).
#[derive(Template)]
#[template(path = "notifications.html")]
struct NotificationsView {
    as_of: String,
    version: u64,
    none: &'static str,
    /// Each row's time, project and words.
    rows: Vec<(String, String, String)>,
    shown: usize,
    problem: Option<String>,
}

/// The doctor's page (EP-31).
#[derive(Template)]
#[template(path = "diagnostics.html")]
struct DiagnosticsView {
    as_of: String,
    version: u64,
    said: Option<SaidView>,
    gathered: Option<String>,
    /// The real window: the page offers Check again (EP-31, the owner's choice).
    live: bool,
    checking: bool,
    none: &'static str,
    groups: Vec<DoctorGroup>,
}

/// One group of the doctor's report: its title, and each line's level word, its class, its name
/// and its words.
struct DoctorGroup {
    title: &'static str,
    lines: Vec<(&'static str, &'static str, String, String)>,
}

/// The class of a session's state in words: at work, waiting, ended well, deferred, ended badly.
fn state_class(state: &str) -> &'static str {
    match state {
        "running" => "busy",
        "waiting for an answer" => "warn",
        "ended: completed" => "good",
        "ended: deferred" => "warn",
        state if state.starts_with("ended") => "bad",
        _ => "weak",
    }
}

/// `text` as paragraphs split at its blank lines, each escaped, its line breaks kept by the
/// stylesheet (EP-31: the Sessions pane draws a text as it came).
fn paragraphs(text: &str, class: &str) -> String {
    fn flush(lines: &mut Vec<&str>, class: &str, html: &mut String) {
        if !lines.is_empty() {
            html.push_str(&format!(
                "<p class=\"{class}\">{}</p>",
                escape(&lines.join("\n"))
            ));
            lines.clear();
        }
    }
    let mut html = String::new();
    let mut lines = Vec::new();
    for line in text.lines() {
        if line.trim().is_empty() {
            flush(&mut lines, class, &mut html);
        } else {
            lines.push(line);
        }
    }
    flush(&mut lines, class, &mut html);
    html
}

/// One transcript entry as HTML, as the Sessions pane draws it (EP-31): the prompt folded (a
/// resume's open), text as paragraphs, a tool call in one line with its result folded under it,
/// a refusal, a request with its answer, a notice, the result.
fn entry_html(entry: &Entry) -> String {
    match entry {
        Entry::Prompt { text, at, resumed } => {
            // Its time with its day when it is not today (the review's).
            let when = local_time(at.with_timezone(&Utc));
            let (head, class, open) = if *resumed {
                (
                    format!("Resumed at {when} with:"),
                    "prompt resumed",
                    " open",
                )
            } else {
                (format!("Prompt · sent at {when}"), "prompt", "")
            };
            format!(
                "<details class=\"{class}\"{open}><summary>{}</summary>{}</details>",
                escape(&head),
                paragraphs(text, "weak small")
            )
        }
        Entry::Text(text) => paragraphs(text, "text"),
        Entry::Tool { call, result } => {
            let line = format!(
                "<p class=\"tool\"><span class=\"tool-name\">{}</span> <code>{}</code></p>",
                escape(&call.tool),
                escape(&call.summary)
            );
            let Some(result) = result else {
                return line;
            };
            let (head, class) = if result.is_error {
                (
                    format!("failed · {}", transcript::lines(result.lines)),
                    "result failed",
                )
            } else {
                (transcript::lines(result.lines), "result")
            };
            let shown = result.excerpt.lines().count();
            let more = if result.lines > shown {
                format!(
                    "<p class=\"weak small\">… {} more</p>",
                    transcript::lines(result.lines - shown)
                )
            } else {
                String::new()
            };
            format!(
                "{line}<details class=\"{class}\"><summary>{}</summary><pre>{}</pre>{more}</details>",
                escape(&head),
                escape(&result.excerpt)
            )
        }
        Entry::Denied(denied) => format!(
            "<p class=\"bad\">Refused without asking: {} ({})</p>",
            escape(&denied.tool),
            escape(&denied.message)
        ),
        Entry::Asked { request, answer } => {
            let (words, class) = match answer {
                Some(words) => (words.as_str(), "weak"),
                None => ("waiting: answer it above, or on the decisions page", "warn"),
            };
            format!(
                "<p class=\"warn\">Asked the host: {}</p><p class=\"answer small {class}\">→ {}</p>",
                escape(&request.summary),
                escape(words)
            )
        }
        Entry::Notice(text) => format!("<p class=\"warn small\">{}</p>", escape(text)),
        Entry::Result(finish) => {
            let class = if finish.succeeded() {
                "good"
            } else if finish.is_deferred() {
                "warn"
            } else {
                "bad"
            };
            let mut html = String::new();
            if let Some(deferred) = &finish.deferred {
                html.push_str(&format!(
                    "<p class=\"{class}\">Deferred at {}: an answer resumes the session.</p>",
                    escape(&deferred.summary)
                ));
            }
            html.push_str(&format!(
                "<p class=\"result {class}\"><strong>Result: {} · {} turns · {} s · an estimated \
                 ${:.3}</strong></p>",
                escape(&finish.subtype),
                finish.turns,
                finish.duration_ms / 1000,
                finish.cost_usd
            ));
            if let Some(text) = &finish.text {
                html.push_str(&paragraphs(text, "text"));
            }
            for error in &finish.errors {
                html.push_str(&format!("<p class=\"bad\">{}</p>", escape(error)));
            }
            html
        }
    }
}

/// The page of the window's session `id` in `published`, with the outcome of the form that led
/// here; none when the window holds no such session (EP-31).
pub fn session(
    published: &Published,
    id: &str,
    said: Option<&Said>,
) -> Option<Result<String, askama::Error>> {
    let page: &SessionPage = published.session(id)?;
    let path = crate::server::session_path(&page.id);
    let runs = match page.runs {
        1 => String::new(),
        n => format!(" · {n} runs"),
    };
    // The entries the page leaves out are in the tab; those the window let go are nowhere (the
    // review's).
    let in_tab = page.first - page.let_go;
    let earlier = [
        match in_tab {
            0 => String::new(),
            1 => "1 earlier entry is in the window's tab only.".to_owned(),
            n => format!("{n} earlier entries are in the window's tab only."),
        },
        match page.let_go {
            0 => String::new(),
            n => format!("{n} before them were let go to keep the transcript short."),
        },
    ]
    .into_iter()
    .filter(|words| !words.is_empty())
    .collect::<Vec<_>>()
    .join(" ");
    Some(
        SessionView {
            as_of: as_of(published.at),
            version: published.version,
            said: SaidView::of(said),
            id: page.id.clone(),
            title: page.title.clone(),
            project: page.project.clone(),
            project_link: project_link(&page.project),
            state: page.state.clone(),
            state_class: state_class(&page.state),
            facts: format!(
                "{} · {} · started {}{runs}",
                page.model,
                page.effort,
                local_time(page.started)
            ),
            autopilot: page.autopilot.clone(),
            dropped: page.dropped.clone(),
            ended: page.ended.clone(),
            stop: page.stoppable.then(|| (page.key, page.ep.clone())),
            earlier,
            entries: page
                .entries
                .iter()
                .enumerate()
                .map(|(i, entry)| EntryView {
                    n: page.first + i,
                    html: entry_html(entry),
                })
                .collect(),
            streaming: page.streaming.clone(),
            waiting: page
                .waiting
                .iter()
                .map(|asking| AskView::back_to(asking, &path))
                .collect(),
            deferred: page
                .deferred
                .iter()
                .map(|asking| AskView::back_to(asking, &path))
                .collect(),
            path,
        }
        .render(),
    )
}

/// The sessions page of `published` (EP-31).
pub fn sessions(published: &Published) -> Result<String, askama::Error> {
    let list = &published.sessions;
    let outside = match list.outside {
        0 => String::new(),
        1 => "One more session runs outside the registered projects.".to_owned(),
        n => format!("{n} more sessions run outside the registered projects."),
    };
    SessionsView {
        as_of: as_of(published.at),
        version: published.version,
        starting: list.starting,
        window: list
            .window
            .iter()
            .map(|run| {
                let runs = match run.runs {
                    1 => String::new(),
                    n => format!(" · {n} runs"),
                };
                WindowView {
                    link: crate::server::session_path(&run.id),
                    title: run.title.clone(),
                    state: run.state.clone(),
                    class: state_class(&run.state),
                    facts: format!(
                        "{} · {} · started {}{runs}",
                        run.model,
                        run.effort,
                        local_time(run.started)
                    ),
                    last: run.last.clone(),
                    chained: run.chained,
                }
            })
            .collect(),
        elsewhere: list
            .elsewhere
            .iter()
            .map(|seen| SeenView {
                project: seen.project.clone(),
                mode: seen.mode.clone(),
                status: seen.status,
                class: match seen.status {
                    "busy" => "busy",
                    "waiting" => "warn",
                    "done" => "good",
                    "failed" => "bad",
                    _ => "weak",
                },
                waiting: seen.waiting,
                doing: seen.doing.clone(),
                since: seen.started.map(local_time).unwrap_or_default(),
                name: seen.name.clone().unwrap_or_default(),
                figures: seen.figures.join(" · "),
            })
            .collect(),
        outside,
        poll_problem: list.poll_problem.clone(),
        rows: list
            .rows
            .iter()
            .map(|row| {
                let facts: Vec<String> = [&row.short, &row.model, &row.took, &row.cost]
                    .into_iter()
                    .filter(|fact| !fact.is_empty())
                    .cloned()
                    .chain(row.figures.iter().cloned())
                    .collect();
                RowView {
                    when: row
                        .started
                        .map_or_else(|| row.started_at.clone(), local_time),
                    title: row.title.clone(),
                    link: row.page.as_deref().map(crate::server::session_path),
                    mode: row.mode.clone(),
                    state: row.state.clone(),
                    class: match row.state.as_str() {
                        "completed" => "good",
                        "deferred" => "warn",
                        "failed" | "stopped" => "bad",
                        _ => "weak",
                    },
                    facts: facts.join(" · "),
                    result: row.result.clone(),
                }
            })
            .collect(),
        problem: list.problem.clone(),
    }
    .render()
}

/// The notifications page of `published` (EP-31).
pub fn notifications(published: &Published) -> Result<String, askama::Error> {
    let notifications = &published.notifications;
    NotificationsView {
        as_of: as_of(published.at),
        version: published.version,
        none: if notifications.read {
            "None yet: the window logs each notification and what became of it, each Show and \
             each badge."
        } else {
            "The window keeps no ledger here, so it lists no notification."
        },
        rows: notifications
            .rows
            .iter()
            .map(|row| {
                (
                    row.at.map(local_time).unwrap_or_default(),
                    row.project.clone().unwrap_or_default(),
                    row.words.clone(),
                )
            })
            .collect(),
        shown: published::NOTIFICATIONS,
        problem: notifications.problem.clone(),
    }
    .render()
}

/// The doctor's page of `published`, with the outcome of the form that led here (EP-31).
pub fn diagnostics(published: &Published, said: Option<&Said>) -> Result<String, askama::Error> {
    let doctor = &published.doctor;
    let now = Utc::now();
    let groups = doctor
        .report
        .as_ref()
        .map(|(report, _)| {
            Group::ALL
                .into_iter()
                .filter_map(|group| {
                    let lines: Vec<_> = report
                        .group(group)
                        .map(|line| {
                            let class = match line.level {
                                Level::Ok => "good",
                                Level::Note => "weak",
                                Level::Warn => "warn",
                                Level::Fail => "bad",
                            };
                            (
                                line.level.word(),
                                class,
                                line.name.clone(),
                                line.text.clone(),
                            )
                        })
                        .collect();
                    (!lines.is_empty()).then(|| DoctorGroup {
                        title: group.title(),
                        lines,
                    })
                })
                .collect()
        })
        .unwrap_or_default();
    DiagnosticsView {
        as_of: as_of(published.at),
        version: published.version,
        said: SaidView::of(said),
        gathered: doctor
            .report
            .as_ref()
            .map(|(_, at)| format!("Gathered at {}, {}.", local_time(*at), ago(now, *at))),
        live: doctor.live,
        checking: doctor.checking,
        none: if doctor.live {
            "No report yet: the window gathers one when its Diagnostics pane is first shown, and \
             at Check again here or in the pane."
        } else {
            "This window gathers no report: a screenshot run or a test."
        },
        groups,
    }
    .render()
}

/// The list page of `published`, with the outcome of the Pause or Resume that led here.
pub fn list(published: &Published, said: Option<&Said>) -> Result<String, askama::Error> {
    let list = &published.list;
    let projects: Vec<&Line> = list.projects().collect();
    let owner_items: usize = projects.iter().map(|line| line.owner_items).sum();
    let messages: usize = projects.iter().map(|line| line.messages).sum();
    let summary = format!(
        "{} · {} · {}",
        plural(projects.len(), "project"),
        plural(owner_items, "owner task"),
        plural(messages, "inbox message")
    );
    let decisions = match published.decisions.count() {
        0 => "Nothing waits for an answer".to_owned(),
        1 => "1 waits for an answer".to_owned(),
        n => format!("{n} wait for an answer"),
    };
    ListPage {
        as_of: as_of(published.at),
        version: published.version,
        said: SaidView::of(said),
        usage: UsageView::of(&list.usage, Utc::now()),
        autopilot: AutopilotView::of(&list.autopilot),
        summary,
        decisions,
        problem: list.problem.clone(),
        reading: list.reading,
        groups: list
            .groups
            .iter()
            .map(|group| GroupView {
                heading: group.heading(),
                open: !matches!(group.state, State::Done | State::Archived),
                projects: group.projects.iter().map(LineView::of).collect(),
            })
            .collect(),
        unadopted: list.unadopted.clone(),
    }
    .render()
}

/// A page of one heading and one line: a refusal, a page not found, the window not serving.
pub fn message(heading: &str, words: &str) -> String {
    MessagePage { heading, words }
        .render()
        .unwrap_or_else(|_| format!("{heading}: {words}"))
}

/// `1 project`, `3 projects`.
fn plural(count: usize, word: &str) -> String {
    if count == 1 {
        format!("1 {word}")
    } else {
        format!("{count} {word}s")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::published::{Badge, Decisions, Group, List, Mail, Message, Project};
    use crw_core::roadmap::table::Counts;

    fn line(name: &str, state: State, next: Next) -> Line {
        Line {
            name: name.to_owned(),
            state,
            pinned: false,
            proposes: false,
            autopilot: false,
            next,
            counts: Counts {
                done: 3,
                total: 8,
                stretch: 1,
            },
            owner_items: 0,
            messages: 0,
            urgent: 0,
            sessions: Vec::new(),
            usage_tenths: None,
            badges: Vec::new(),
        }
    }

    fn published(groups: Vec<Group>) -> Published {
        Published::of(
            published::Parts {
                list: List {
                    reading: false,
                    problem: None,
                    groups,
                    unadopted: vec!["new-folder".to_owned()],
                    ..List::default()
                },
                decisions: Decisions::default(),
                ..published::Parts::default()
            },
            Utc::now(),
            3,
        )
    }

    /// A project's page data: a note with a label and parts, a task, a gated task, an older
    /// form, an urgent message.
    fn harbor_page() -> Project {
        Project {
            name: "harbor & co".to_owned(),
            state: State::Active,
            folder: true,
            next: Next::Brief {
                id: "EP-4".to_owned(),
                size: Some("M".to_owned()),
                title: "The `scan` view".to_owned(),
            },
            note: Some(
                "The view's rows. **From EP-3:** (1) the `crw scan` table <b>; (2) its sort."
                    .to_owned(),
            ),
            waiting: vec![
                Item {
                    id: Some("OT-3".to_owned()),
                    gate: None,
                    text: "Look at the **real** list.".to_owned(),
                    meta: "roadmap/owner-tasks.md:5 · EP-2, 2026-11-01".to_owned(),
                    tickable: true,
                },
                Item {
                    id: Some("R-2".to_owned()),
                    gate: Some("before EP-5".to_owned()),
                    text: "**R-2** An older risk.".to_owned(),
                    meta: "risks list · roadmap/README.md:40".to_owned(),
                    tickable: false,
                },
            ],
            resolved: 4,
            keeps_tasks: true,
            uncounted: vec![Item {
                id: None,
                gate: None,
                text: "A parked block.".to_owned(),
                meta: "parked block · roadmap/EP-3-x.md:9".to_owned(),
                tickable: false,
            }],
            inbox: Some(Mail {
                open: vec![Message {
                    id: "IN-2".to_owned(),
                    from: "beta EP-7, 2026-11-02".to_owned(),
                    urgent: true,
                    text: "The `lock` drifted.".to_owned(),
                }],
                total: 3,
            }),
            sessions: vec!["a headless EP-4 started here at 10:00 is working".to_owned()],
            busy: Some("a headless run of the window's works there".to_owned()),
            revert: false,
            pinned: false,
            proposes: false,
            autopilot: false,
            runs: Vec::new(),
        }
    }

    #[test]
    fn a_projects_page_draws_its_note_items_inbox_and_forms() {
        let mut page = published(Vec::new());
        page.projects.push(harbor_page());
        let said: Said = Ok("Ticked OT-3 in harbor & co.".to_owned());
        let html = project(&page, "HARBOR & CO", Some(&said)).unwrap().unwrap();
        // The name escaped, the link's segment encoded, the outcome's line.
        assert!(
            html.contains("<h1 class=\"name\">harbor &#38; co</h1>")
                || html.contains("<h1 class=\"name\">harbor &amp; co</h1>"),
            "{html}"
        );
        assert!(
            html.contains(r#"action="/m/p/harbor%20%26%20co/tick""#),
            "{html}"
        );
        assert!(
            html.contains(r#"<p class="said done">Ticked OT-3 in harbor &#38; co.</p>"#)
                || html.contains(r#"<p class="said done">Ticked OT-3 in harbor &amp; co.</p>"#),
            "{html}"
        );
        // The note in blocks: its paragraph, its label, its numbered parts, escaped and marked.
        assert!(html.contains("<p>The view&#39;s rows.</p>"), "{html}");
        assert!(
            html.contains(r#"<p class="label">From EP-3:</p>"#),
            "{html}"
        );
        assert!(
            html.contains(
                r#"<p class="part"><span class="n">(1)</span> the <code>crw scan</code> table &lt;b&gt;;</p>"#
            ),
            "{html}"
        );
        assert!(!html.contains(" <b>;"), "the note's text is escaped");
        // The task's form names its id; the older form is read-only with its gate.
        assert!(html.contains(r#"<input type="hidden" name="id" value="OT-3">"#));
        assert!(html.contains("<button type=\"submit\">Tick OT-3</button>"));
        assert!(!html.contains("value=\"R-2\""), "an older form has no form");
        assert!(html.contains(r#"<span class="mark warn">before EP-5</span>"#));
        assert!(html.contains("An older form: changed only in its file."));
        assert!(html.contains("2 open, 4 resolved"));
        assert!(html.contains("Risks and older forms"));
        assert!(html.contains("the project keeps owner-tasks.md, which is what waits"));
        // The inbox: the urgent message with its disposition's form; the compose form.
        assert!(html.contains("1 open of 3"));
        assert!(html.contains(r#"<span class="mark bad">urgent</span>"#));
        assert!(html.contains(r#"action="/m/p/harbor%20%26%20co/message""#));
        assert!(html.contains(r#"name="disposition" required"#));
        assert!(html.contains(r#"action="/m/p/harbor%20%26%20co/send""#));
        assert!(html.contains(r#"<textarea name="text""#));
        assert!(
            html.contains("Now a headless run of the window&#39;s works there"),
            "{html}"
        );
        assert!(html.contains("a headless EP-4 started here at 10:00 is working"));
        // Nothing from outside, no script but the tool's own (EP-30b).
        assert!(!html.contains("http://") && !html.contains("https://"));
        assert_eq!(html.matches("<script").count(), 1);
        assert!(html.contains(r#"<script src="/m/app.js" defer></script>"#));
        // A project the window does not list.
        assert!(project(&page, "nowhere", None).is_none());
        // A failure's line, and no form where the folder is missing.
        let mut gone = harbor_page();
        gone.folder = false;
        page.projects = vec![gone];
        let failed: Said = Err("harbor & co: its folder is missing.".to_owned());
        let html = project(&page, "harbor & co", Some(&failed))
            .unwrap()
            .unwrap();
        assert!(html.contains(r#"<p class="said failed">"#));
        assert!(!html.contains("<form"), "{html}");
        // A project without owner-tasks.md: nothing counted, its older forms said to be so
        // (EP-50's review).
        let mut bare = harbor_page();
        bare.keeps_tasks = false;
        bare.waiting.clear();
        page.projects = vec![bare];
        let html = project(&page, "harbor & co", None).unwrap().unwrap();
        assert!(
            html.contains("Nothing counted: the project keeps no owner-tasks.md."),
            "{html}"
        );
        assert!(html.contains("the project keeps no owner-tasks.md, so nothing here is counted"));
        assert!(!html.contains("which is what waits on the owner"), "{html}");
        // A revert open there: its words in place of every form (the review's).
        let mut reverting = harbor_page();
        reverting.revert = true;
        page.projects = vec![reverting];
        let html = project(&page, "harbor & co", None).unwrap().unwrap();
        assert!(
            html.contains("A revert is open here in the window"),
            "{html}"
        );
        assert!(!html.contains("<form"), "{html}");
        // The list line's marks beside the state, from the project's own part (EP-30b).
        let mut pinned = harbor_page();
        pinned.pinned = true;
        pinned.autopilot = true;
        page.projects = vec![pinned];
        let html = project(&page, "harbor & co", None).unwrap().unwrap();
        assert!(
            html.contains(r#"<span class="mark weak">pinned</span>"#),
            "{html}"
        );
        assert!(html.contains(r#"<span class="mark good">autopilot on</span>"#));
    }

    #[test]
    fn the_decisions_page_draws_each_request_as_its_form() {
        use crw_core::session::{Choice, Question, Request};
        let question = |text: &str, header: &str, several: bool| Question {
            text: text.to_owned(),
            header: header.to_owned(),
            options: vec![
                Choice {
                    label: "Top <1>".to_owned(),
                    description: Some("the upper shelf".to_owned()),
                },
                Choice {
                    label: "Bottom".to_owned(),
                    description: None,
                },
            ],
            multi_select: several,
        };
        let asked = Request {
            id: "req-1".to_owned(),
            tool: "AskUserQuestion".to_owned(),
            summary: "Which shelf?".to_owned(),
            reason: None,
            questions: vec![
                question("Which shelf?", "Shelf", false),
                question("Which rows?", "Rows", true),
            ],
            input: String::new(),
        };
        let push = Request {
            id: "req-2".to_owned(),
            tool: "Bash".to_owned(),
            summary: "Bash git push".to_owned(),
            reason: Some("A rule asks first".to_owned()),
            questions: Vec::new(),
            input: "{\"command\":\"git push\"}".to_owned(),
        };
        let at = Utc::now();
        let mut page = published(Vec::new());
        page.decisions = Decisions {
            waiting: vec![
                Asking {
                    key: 7,
                    title: "harbor EP-4".to_owned(),
                    project: "harbor".to_owned(),
                    request: asked,
                    at,
                    how: How::Live { limit: Some(30) },
                },
                Asking {
                    key: 7,
                    title: "harbor EP-4".to_owned(),
                    project: "harbor".to_owned(),
                    request: push.clone(),
                    at,
                    how: How::Live { limit: None },
                },
            ],
            deferred: vec![Asking {
                key: 9,
                title: "meadow EP-2".to_owned(),
                project: "meadow".to_owned(),
                request: push,
                at,
                how: How::Deferred { hook: true },
            }],
            elsewhere: vec![crate::published::Elsewhere {
                project: "orchard".to_owned(),
                mode: "visual",
                words: "waits for you in VS Code: a permission prompt".to_owned(),
                name: None,
            }],
        };
        let said: Said =
            Err("Already answered: the session no longer waits on that request.".to_owned());
        let html = decisions(&page, Some(&said)).unwrap();
        assert!(
            html.contains(r#"<p class="said failed">Already answered"#),
            "{html}"
        );
        // The question's form: its session and request, a radio for one answer, boxes for
        // several, the owner's own words, the two buttons.
        assert!(html.contains(r#"<input type="hidden" name="key" value="7">"#));
        assert!(html.contains(r#"<input type="hidden" name="request" value="req-1">"#));
        assert!(
            html.contains(r#"<input type="radio" name="pick0" value="0"> Top &lt;1&gt;"#)
                || html.contains(r#"<input type="radio" name="pick0" value="0"> Top &#60;1&#62;"#),
            "{html}"
        );
        assert!(
            html.contains(r#"<input type="checkbox" name="pick1" value="1"> Bottom"#),
            "{html}"
        );
        assert!(html.contains(r#"name="other0""#) && html.contains(r#"name="other1""#));
        assert!(html.contains(r#"value="answer">Send answers</button>"#));
        assert!(html.contains(r#"value="decline" class="deny">Decline</button>"#));
        assert!(html.contains("the upper shelf"));
        assert!(
            html.contains("deferred at"),
            "the time limit's moment: {html}"
        );
        // The permission's form, and the deferred run's, whose buttons resume it.
        assert!(html.contains("<code>Bash git push</code>"));
        assert!(html.contains(r#"value="allow">Allow</button>"#));
        assert!(html.contains(r#"value="allow">Allow and resume</button>"#));
        assert!(html.contains("by the defer route&#39;s hook"), "{html}");
        assert!(html.contains("A rule asks first"));
        // The session waiting in VS Code, read-only.
        assert!(html.contains("It waits for you in VS Code: a permission prompt."));
        assert_eq!(html.matches("<form").count(), 3, "no form for VS Code's");
        assert!(html.contains(r#"action="/m/decisions/answer""#));
        // Nothing waits: the page says so.
        page.decisions = Decisions::default();
        let html = decisions(&page, None).unwrap();
        assert!(html.contains("Nothing waits for you."));
        assert!(!html.contains("<form"));
    }

    #[test]
    fn the_list_links_each_project_and_counts_the_decisions() {
        let mut page = published(vec![Group {
            state: State::Active,
            projects: vec![line(
                "my project",
                State::Active,
                Next::None {
                    words: "Every row is checked".to_owned(),
                    missing: false,
                },
            )],
        }]);
        page.decisions.elsewhere.push(crate::published::Elsewhere {
            project: "my project".to_owned(),
            mode: "visual",
            words: "waits for you in VS Code".to_owned(),
            name: None,
        });
        let html = list(&page, None).unwrap();
        assert!(
            html.contains(r#"<a class="name" href="/m/p/my%20project">my project</a>"#),
            "{html}"
        );
        assert!(html.contains("1 waits for an answer"), "{html}");
        assert!(html.contains(r#"href="/m/decisions""#));
    }

    #[test]
    fn the_list_page_draws_each_project_escaped_with_its_titles_markup() {
        let mut harbor = line(
            "harbor <b>",
            State::Active,
            Next::Brief {
                id: "EP-4".to_owned(),
                size: Some("M".to_owned()),
                title: "The `crw scan` view & **its** rows".to_owned(),
            },
        );
        harbor.pinned = true;
        harbor.autopilot = true;
        harbor.owner_items = 2;
        harbor.messages = 3;
        harbor.urgent = 1;
        harbor.sessions = vec!["a headless EP-4 started here at 10:00 is working".to_owned()];
        harbor.badges = vec![Badge {
            column: "Sync",
            word: "↑2 not pushed".to_owned(),
            sentence: "2 commits not pushed: the origin checked 5 min ago".to_owned(),
            tone: "warn",
            clocked: false,
        }];
        let done = line(
            "orchard",
            State::Done,
            Next::None {
                words: "Every row is checked".to_owned(),
                missing: false,
            },
        );
        let page = list(
            &published(vec![
                Group {
                    state: State::Active,
                    projects: vec![harbor],
                },
                Group {
                    state: State::Done,
                    projects: vec![done],
                },
            ]),
            None,
        )
        .unwrap();
        assert!(
            page.contains("harbor &#60;b&#62;") || page.contains("harbor &lt;b&gt;"),
            "{page}"
        );
        assert!(!page.contains("harbor <b>"), "the name is escaped");
        assert!(page.contains("<code>crw scan</code>"), "{page}");
        assert!(page.contains("<strong>its</strong>"), "{page}");
        assert!(
            page.contains("view &#38; ") || page.contains("view &amp; "),
            "{page}"
        );
        assert!(page.contains(r#"<span class="ep">EP-4</span>"#));
        assert!(
            page.contains(
                r#"<progress value="3" max="8"></progress> 3 of 8 rows done (38 %), 1 stretch"#
            ),
            "{page}"
        );
        assert!(page.contains("2 owner tasks"));
        assert!(page.contains("3 inbox messages, 1 urgent"));
        assert!(page.contains("a headless EP-4 started here at 10:00 is working"));
        assert!(
            page.contains(r#"<span class="badge warn">↑2 not pushed</span>"#),
            "{page}"
        );
        assert!(
            page.contains("<dt>Sync</dt><dd>2 commits not pushed"),
            "{page}"
        );
        assert!(page.contains("pinned") && page.contains("autopilot on"));
        assert!(page.contains("Every row is checked"));
        // Finished work folded, active work open, the unadopted folder named.
        // The fold is the group's own `<details>`, whose `open` the Done group lacks (the
        // review's: a test of the text before the heading alone could not fail).
        assert!(
            page.contains("<details class=\"group\" open>\n<summary><h2>Active"),
            "{page}"
        );
        assert!(
            page.contains("<details class=\"group\">\n<summary><h2>Done"),
            "the Done group is folded: {page}"
        );
        assert!(page.contains("<li>new-folder</li>"));
        assert!(page.contains("2 projects · 2 owner tasks · 3 inbox messages"));
        // Nothing from outside the window's own server.
        assert!(
            !page.contains("http://") && !page.contains("https://"),
            "{page}"
        );
        assert!(page.contains(r#"href="/m/app.css""#));
        // The one script is the tool's own (EP-30b).
        assert_eq!(page.matches("<script").count(), 1);
        assert!(page.contains(r#"<script src="/m/app.js" defer></script>"#));
    }

    #[test]
    fn a_list_not_read_yet_says_so_without_a_time() {
        let mut empty = published(Vec::new());
        empty.at = DateTime::<Utc>::MIN_UTC;
        empty.list.reading = true;
        empty.list.unadopted.clear();
        let page = list(&empty, None).unwrap();
        assert!(page.contains("Reading the projects…"));
        assert!(!page.contains("as of"));
        assert!(page.contains("0 projects"));
    }

    /// The list's usage strip and autopilot block (EP-30b): each figure with its reset drawn
    /// now, the Pause or the Resume as a said value, the held chains, the plans, the log; the
    /// list listens to its own path and the decisions', and loads the script.
    #[test]
    fn the_list_draws_the_usage_strip_and_the_autopilot_block() {
        let now = Utc::now();
        let mut page = published(Vec::new());
        page.list.usage = published::Usage {
            on: true,
            figures: vec![
                published::Figure {
                    name: "Week",
                    percent: 41.0,
                    resets_at: Some(
                        now + chrono::TimeDelta::hours(51) + chrono::TimeDelta::seconds(30),
                    ),
                    at: now - chrono::TimeDelta::minutes(3),
                    from: "the usage endpoint",
                },
                published::Figure {
                    name: "Fable",
                    percent: 96.0,
                    resets_at: None,
                    at: now,
                    from: "the usage endpoint",
                },
            ],
            session_closed: true,
            degraded: Some("the endpoint did not answer: timed out".to_owned()),
        };
        page.list.autopilot = published::Autopilot {
            paused: true,
            held_by_usage: Vec::new(),
            on: vec!["harbor".to_owned()],
            daily_total: 8,
            chained_today: Some(3),
            held: vec![(
                "harbor".to_owned(),
                "held after EP-3 until the pause lifts: <your Pause>".to_owned(),
            )],
            plans: vec![(
                "harbor".to_owned(),
                "after EP-3: EP-4 fable at high (the rules' choice)".to_owned(),
                "70 % of the session (five-hour) window".to_owned(),
            )],
            log: vec![published::LogLine {
                at: Some(now),
                project: Some("harbor".to_owned()),
                words: "chained EP-4 after EP-3".to_owned(),
            }],
            problem: None,
        };
        let said: Said = Ok("The autopilot is paused already.".to_owned());
        let html = list(&page, Some(&said)).unwrap();
        assert!(
            html.contains(r#"<main data-listen="/m/ /m/decisions" data-version="3">"#),
            "{html}"
        );
        assert!(html.contains(r#"<script src="/m/app.js" defer></script>"#));
        assert!(html.contains(r#"<p class="said done">The autopilot is paused already.</p>"#));
        for part in ["summary", "usage", "autopilot", "projects"] {
            assert!(html.contains(&format!(r#"data-part="{part}""#)), "{part}");
        }
        assert!(
            html.contains(r#"<span class="share good">41 %</span>"#),
            "{html}"
        );
        assert!(html.contains(r#"<span class="share bad">96 %</span>"#));
        assert!(html.contains("resets in 2 d 3 h, "), "{html}");
        assert!(
            html.contains("read 3 min ago from the usage endpoint"),
            "{html}"
        );
        assert!(html.contains("no reset time given"));
        assert!(html.contains("Session (five-hour window): no window open."));
        assert!(html.contains("Degraded: the endpoint did not answer: timed out."));
        // Paused: the form resumes, by a said value.
        assert!(html.contains(r#"<span class="mark warn">paused by you</span>"#));
        assert!(html.contains(r#"action="/m/autopilot""#));
        assert!(html.contains(r#"<input type="hidden" name="pause" value="off">"#));
        assert!(html.contains("<button type=\"submit\">Resume</button>"));
        assert!(html.contains("on in harbor"));
        assert!(
            html.contains("held after EP-3 until the pause lifts: &#60;your Pause&#62;")
                || html.contains("held after EP-3 until the pause lifts: &lt;your Pause&gt;"),
            "escaped: {html}"
        );
        assert!(html.contains("this chain holds at 70 % of the session (five-hour) window."));
        assert!(html.contains("chained EP-4 after EP-3"));
        // The day's line (EP-31b), the section's words, read-only: no form sets the total.
        assert!(
            html.contains(
                r#"<p class="small">3 of 8 chained runs today in all projects; a chain ends at this total or at its project"#
            ),
            "{html}"
        );
        assert!(html.contains("own runs a day, whichever it meets first.</p>"));
        assert!(!html.contains("The total is reached"));
        assert!(!html.contains("autopilot_daily_total"), "no setting's form");
        assert_eq!(html.matches("<form").count(), 1, "the Pause's alone");
        // Not paused: the form pauses.
        page.list.autopilot.paused = false;
        let html = list(&page, None).unwrap();
        assert!(html.contains(r#"<input type="hidden" name="pause" value="on">"#));
        assert!(html.contains("<button type=\"submit\">Pause</button>"));
        assert!(html.contains(r#"<span class="mark good">on</span>"#));
        // The total reached: the line warns; a count that did not read says so.
        page.list.autopilot.chained_today = Some(8);
        let html = list(&page, None).unwrap();
        assert!(
            html.contains(r#"<p class="small warn">8 of 8 chained runs today in all projects; "#),
            "{html}"
        );
        assert!(
            html.contains("The total is reached for today: a chain ends at its next gate.</p>")
        );
        page.list.autopilot.chained_today = None;
        let html = list(&page, None).unwrap();
        assert!(
            html.contains(r#"<p class="small">8 chained runs a day in all projects (today"#),
            "{html}"
        );
        // The meter off: the strip says so.
        page.list.usage = published::Usage::default();
        assert!(
            list(&page, None)
                .unwrap()
                .contains("The usage meter is off in this window.")
        );
    }

    /// A project's page (EP-30b): its runs, a Stop for each that works, the link to its start
    /// page; it listens to its own path.
    #[test]
    fn a_projects_page_lists_its_runs_with_stop_and_links_its_start() {
        let mut page = published(Vec::new());
        let mut harbor = harbor_page();
        harbor.runs = vec![
            published::Run {
                key: 7,
                ep: "EP-4".to_owned(),
                session: "3c9d2e71-5b04-4f8a-9e61-0a7b8c9d1e2f".to_owned(),
                state: "running".to_owned(),
                started: Utc::now(),
                stoppable: true,
                chained: true,
            },
            published::Run {
                key: 8,
                ep: "EP-3".to_owned(),
                session: "0f1e2d3c-0000-4000-8000-000000000002".to_owned(),
                state: "stopping".to_owned(),
                started: Utc::now(),
                stoppable: false,
                chained: false,
            },
        ];
        page.projects.push(harbor);
        let html = project(&page, "harbor & co", None).unwrap().unwrap();
        assert!(
            html.contains(r#"<main data-listen="/m/p/harbor%20%26%20co" data-version="3">"#),
            "{html}"
        );
        assert!(
            html.contains(
                r#"<a class="action" href="/m/p/harbor%20%26%20co/start">Start EP-4 headless…</a>"#
            ),
            "{html}"
        );
        assert!(html.contains(r#"action="/m/p/harbor%20%26%20co/stop""#));
        assert!(html.contains(r#"<input type="hidden" name="key" value="7">"#));
        assert!(html.contains(r#"<input type="hidden" name="ep" value="EP-4">"#));
        assert!(html.contains("Stop EP-4</button>"));
        assert!(html.contains(
            r#"<input type="hidden" name="session" value="3c9d2e71-5b04-4f8a-9e61-0a7b8c9d1e2f">"#
        ));
        assert!(
            !html.contains("Stop EP-3"),
            "a run being stopped has no Stop"
        );
        assert!(html.contains(r#"<span class="mark good">chained</span>"#));
        assert!(html.contains("stopping · since "));
        for part in [
            "head",
            "next",
            "sessions",
            "waiting",
            "uncounted",
            "inbox",
            "send",
        ] {
            assert!(html.contains(&format!(r#"data-part="{part}""#)), "{part}");
        }
        // No start link while a revert is open there, or with no next brief.
        let mut reverting = harbor_page();
        reverting.revert = true;
        page.projects = vec![reverting];
        let html = project(&page, "harbor & co", None).unwrap().unwrap();
        assert!(!html.contains("/start"), "{html}");
    }

    /// The start page (EP-30b): the brief and the rules' choices with their reasons, what the run
    /// is, the notes, and its button with the brief it shows; refused, the reasons in its place.
    #[test]
    fn the_start_page_draws_its_choices_and_its_button_or_its_refusals() {
        let mut start = published::Start {
            project: "harbor & co".to_owned(),
            next: Next::Brief {
                id: "EP-4".to_owned(),
                size: Some("M".to_owned()),
                title: "The `scan` view".to_owned(),
            },
            model: (
                "claude-opus-5-5".to_owned(),
                "rule 1: size S or M".to_owned(),
            ),
            effort: ("max".to_owned(), "rule 1: size S or M".to_owned()),
            ultracode: false,
            permission_mode: (
                "auto".to_owned(),
                "the project's launch defaults".to_owned(),
            ),
            downshift: None,
            run: vec!["It commits itself in the house pattern <and never pushes>.".to_owned()],
            notes: vec!["Owner tasks due first: OT-3.".to_owned()],
            refusals: Vec::new(),
        };
        let html = super::start(&start, Utc::now(), 7).unwrap();
        assert!(
            html.contains(r#"<main data-listen="/m/p/harbor%20%26%20co" data-version="7">"#),
            "{html}"
        );
        assert!(html.contains(r#"<form method="post" action="/m/p/harbor%20%26%20co/start""#));
        assert!(html.contains(r#"<input type="hidden" name="ep" value="EP-4">"#));
        assert!(html.contains("Start EP-4 headless</button>"));
        assert!(html.contains("<code>claude-opus-5-5</code>"));
        assert!(html.contains("rule 1: size S or M"));
        assert!(html.contains("<code>auto</code>"));
        assert!(html.contains("<code>scan</code>"));
        assert!(
            html.contains("&#60;and never pushes&#62;")
                || html.contains("&lt;and never pushes&gt;"),
            "escaped: {html}"
        );
        assert!(html.contains("Owner tasks due first: OT-3."));
        assert!(html.contains(r#"data-part="start""#));
        // Refused: no form, the reasons said.
        start.refusals = vec![
            "the window already has a headless EP-4 started here at 10:00 is running".to_owned(),
            "claude has not answered yet".to_owned(),
        ];
        let html = super::start(&start, Utc::now(), 7).unwrap();
        assert!(!html.contains("<form"), "{html}");
        assert!(html.contains(
            "Not now: the window already has a headless EP-4 started here at 10:00 is running; \
             claude has not answered yet."
        ));
        // Nothing to launch.
        start.next = Next::None {
            words: "nothing to launch: every row is checked".to_owned(),
            missing: false,
        };
        start.refusals = vec!["nothing to launch: every row is checked".to_owned()];
        let html = super::start(&start, Utc::now(), 7).unwrap();
        assert!(!html.contains("<form") && !html.contains("<dl class=\"choices\">"));
    }

    /// The script (EP-30b): it listens to the event stream, names nothing outside the window's
    /// own server, and puts in parts parsed from the page, never text set as markup.
    #[test]
    fn the_script_listens_to_the_stream_and_loads_nothing_from_outside() {
        assert!(SCRIPT.contains(r#"new EventSource("/m/events" + since)"#));
        assert!(SCRIPT.contains("data-version") && SCRIPT.contains(r#"event.data === "*""#));
        // A stream the proxy's answer closed for good is opened anew (the review's).
        assert!(SCRIPT.contains("EventSource.CLOSED") && SCRIPT.contains("setTimeout(listen"));
        assert!(SCRIPT.contains("data-listen") && SCRIPT.contains("data-part"));
        assert!(SCRIPT.contains("DOMParser"));
        for setter in [
            "innerHTML = ",
            "innerHTML=",
            "innerHTML +=",
            "outerHTML",
            "insertAdjacentHTML",
            "document.write",
        ] {
            assert!(
                !SCRIPT.contains(setter),
                "markup is never set from text: {setter}"
            );
        }
        assert!(!SCRIPT.contains("http://") && !SCRIPT.contains("https://"));
        assert!(!SCRIPT.contains("eval("));
    }

    #[test]
    fn a_message_page_escapes_its_words() {
        let page = message("Not allowed", "<script>x</script>");
        assert!(page.contains("<h1>Not allowed</h1>"));
        assert!(!page.contains("<script>"));
    }

    #[test]
    fn the_stylesheet_follows_the_phones_theme() {
        assert!(STYLESHEET.contains("prefers-color-scheme: dark"));
        assert!(!STYLESHEET.contains("url("), "nothing loaded from outside");
    }

    /// A session's page with each kind of entry, a waiting request and its Stop (EP-31).
    fn session_page() -> SessionPage {
        use crw_core::session::{Denied, Finish, Question, Request, ToolResult, ToolUse};
        let request = Request {
            id: "req-1".to_owned(),
            tool: "AskUserQuestion".to_owned(),
            summary: "Which shelf <top>?".to_owned(),
            reason: None,
            questions: vec![Question {
                text: "Which shelf?".to_owned(),
                header: "Shelf".to_owned(),
                options: Vec::new(),
                multi_select: false,
            }],
            input: String::new(),
        };
        SessionPage {
            id: "3c9d2e71-5b04-4f8a-9e61-0a7b8c9d1e2f".to_owned(),
            key: 7,
            project: "harbor & co".to_owned(),
            ep: "EP-4".to_owned(),
            title: "harbor & co EP-4".to_owned(),
            state: "waiting for an answer".to_owned(),
            running: true,
            stoppable: true,
            model: "claude-opus-5-5".to_owned(),
            effort: "max".to_owned(),
            started: Utc::now(),
            runs: 2,
            autopilot: Some("chained by the autopilot · M's cap of 1 h 30 min".to_owned()),
            dropped: Vec::new(),
            ended: None,
            entries: vec![
                Entry::Prompt {
                    text: "Run EP-4.".to_owned(),
                    at: chrono::Local::now(),
                    resumed: false,
                },
                Entry::Text("Reading <the> brief.\n\nThen the tests.".to_owned()),
                Entry::Tool {
                    call: ToolUse {
                        id: "t1".to_owned(),
                        tool: "Bash".to_owned(),
                        summary: "Bash cargo test --all".to_owned(),
                    },
                    result: Some(ToolResult {
                        tool_use_id: "t1".to_owned(),
                        is_error: true,
                        lines: 40,
                        excerpt: "error: <boom>".to_owned(),
                    }),
                },
                Entry::Denied(Denied {
                    tool: "Edit".to_owned(),
                    message: "a neighbour's file".to_owned(),
                    reason: None,
                }),
                Entry::Asked {
                    request: request.clone(),
                    answer: None,
                },
                Entry::Notice("the job was <killed>".to_owned()),
                Entry::Prompt {
                    text: "Go on <now>.".to_owned(),
                    at: chrono::Local::now(),
                    resumed: true,
                },
                Entry::Result(Finish {
                    subtype: "success".to_owned(),
                    is_error: false,
                    text: Some("Done.".to_owned()),
                    turns: 12,
                    cost_usd: 0.5,
                    duration_ms: 300_000,
                    stop_reason: None,
                    deferred: None,
                    denials: 0,
                    errors: vec!["an <error>".to_owned()],
                }),
            ],
            first: 40,
            let_go: 0,
            streaming: "And now".to_owned(),
            waiting: vec![Asking {
                key: 7,
                title: "harbor & co EP-4".to_owned(),
                project: "harbor & co".to_owned(),
                request,
                at: Utc::now(),
                how: How::Live { limit: None },
            }],
            deferred: None,
        }
    }

    #[test]
    fn a_sessions_page_draws_its_transcript_its_answers_and_its_stop() {
        let mut page = published(Vec::new());
        page.runs.push(session_page());
        let id = "3c9d2e71-5b04-4f8a-9e61-0a7b8c9d1e2f";
        let path = format!("/m/s/{id}");
        let said: Said = Ok("harbor & co EP-4: answered.".to_owned());
        let html = session(&page, &id.to_uppercase(), Some(&said))
            .unwrap()
            .unwrap();
        assert!(
            html.contains(&format!(r#"<main data-listen="{path}" data-version="3">"#)),
            "{html}"
        );
        assert!(
            html.contains(r#"<p class="said done">harbor &#38; co EP-4: answered.</p>"#)
                || html.contains(r#"<p class="said done">harbor &amp; co EP-4: answered.</p>"#)
        );
        // The transcript, numbered from its place in the run, every run escaped.
        assert!(
            html.contains(r#"data-part="transcript" data-append"#),
            "{html}"
        );
        assert!(html.contains(r#"<div class="entry" data-n="40"><details class="prompt">"#));
        assert!(html.contains(r#"data-n="47""#) && !html.contains(r#"data-n="48""#));
        assert!(html.contains("<p class=\"text\">Reading &lt;the&gt; brief.</p><p class=\"text\">Then the tests.</p>"), "{html}");
        assert!(
            html.contains(
                r#"<p class="tool"><span class="tool-name">Bash</span> <code>Bash cargo test --all</code></p><details class="result failed"><summary>failed · 40 lines</summary><pre>error: &lt;boom&gt;</pre><p class="weak small">… 39 lines more</p></details>"#
            ),
            "{html}"
        );
        assert!(html.contains("Refused without asking: Edit (a neighbour&#39;s file)"));
        assert!(html.contains("Asked the host: Which shelf &lt;top&gt;?"));
        assert!(html.contains("→ waiting: answer it above, or on the decisions page"));
        assert!(html.contains(r#"<p class="warn small">the job was &lt;killed&gt;</p>"#));
        assert!(
            html.contains(r#"<details class="prompt resumed" open><summary>Resumed at "#)
                && html.contains("Go on &lt;now&gt;."),
            "{html}"
        );
        assert!(html.contains(r#"<p class="bad">an &lt;error&gt;</p>"#));
        assert!(
            !html.contains("<error>") && !html.contains("<killed>") && !html.contains("<boom>")
        );
        assert!(html.contains("Result: success · 12 turns · 300 s · an estimated $0.500"));
        assert!(
            html.contains("40 earlier entries are in the window&#39;s tab only.")
                || html.contains("40 earlier entries are in the window&#39;s tab only"),
            "{html}"
        );
        assert!(
            html.contains(r#"<p class="text streaming">And now…</p>"#),
            "{html}"
        );
        // The waiting request's form comes back here, and so does the Stop.
        assert!(html.contains(r#"action="/m/decisions/answer""#));
        assert_eq!(
            html.matches(&format!(
                r#"<input type="hidden" name="back" value="{path}">"#
            ))
            .count(),
            2,
            "{html}"
        );
        assert!(html.contains(r#"action="/m/p/harbor%20%26%20co/stop""#));
        assert!(html.contains(&format!(
            r#"<input type="hidden" name="session" value="{id}">"#
        )));
        assert!(html.contains("Stop EP-4</button>"));
        // Answers only: no box for a message, no Resume; the desk's actions named.
        assert!(!html.contains("<textarea"), "{html}");
        assert!(!html.contains(">Resume<"));
        assert!(html.contains("a message, Resume, Kill, Revert and Commit are the window's"));
        assert!(html.contains("chained by the autopilot"));
        assert!(html.contains("claude-opus-5-5 · max · started "));
        assert!(html.contains(" · 2 runs"));
        assert!(!html.contains("http://") && !html.contains("https://"));
        // An ended run deferred with its request open: no Stop, its form resumes it and comes
        // back here; the entries let go said apart from those in the tab (the review's).
        let mut ended = session_page();
        ended.stoppable = false;
        ended.running = false;
        let mut open = ended.waiting.remove(0);
        open.how = How::Deferred { hook: true };
        ended.deferred = Some(open);
        ended.state = "ended: deferred".to_owned();
        ended.ended = Some("ended deferred · exit 0".to_owned());
        ended.first = 2040;
        ended.let_go = 2000;
        page.runs = vec![ended];
        let html = session(&page, id, None).unwrap().unwrap();
        assert!(!html.contains("/stop"), "{html}");
        assert!(html.contains(r#"<span class="mark warn">ended: deferred</span>"#));
        assert!(html.contains("Answer and resume</button>"), "{html}");
        assert!(html.contains(&format!(
            r#"<input type="hidden" name="back" value="{path}">"#
        )));
        assert!(
            html.contains(
                "40 earlier entries are in the window&#39;s tab only. 2000 before them were let \
                 go to keep the transcript short."
            ),
            "{html}"
        );
        assert!(session(&page, "nowhere", None).is_none());
    }

    #[test]
    fn the_sessions_page_lists_the_windows_those_elsewhere_and_the_ledgers() {
        use crate::published::{LedgerRow, SeenSession, SessionList, WindowSession};
        let id = "3c9d2e71-5b04-4f8a-9e61-0a7b8c9d1e2f";
        let mut page = published(Vec::new());
        page.sessions = SessionList {
            starting: true,
            window: vec![WindowSession {
                id: id.to_owned(),
                title: "harbor EP-4".to_owned(),
                state: "running".to_owned(),
                model: "fable".to_owned(),
                effort: "high".to_owned(),
                started: Utc::now(),
                runs: 1,
                last: "Bash cargo test".to_owned(),
                chained: true,
            }],
            elsewhere: vec![SeenSession {
                project: "meadow".to_owned(),
                mode: "visual".to_owned(),
                status: "waiting",
                waiting: true,
                doing: "waits for you in VS Code: a permission prompt".to_owned(),
                started: Some(Utc::now() - chrono::TimeDelta::days(2)),
                name: Some("The <berth> list".to_owned()),
                figures: vec!["session 41d8e2c0".to_owned()],
            }],
            outside: 2,
            poll_problem: None,
            rows: vec![
                LedgerRow {
                    started: Some(Utc::now()),
                    started_at: String::new(),
                    short: "3c9d2e71".to_owned(),
                    title: "harbor EP-4".to_owned(),
                    mode: "headless".to_owned(),
                    model: "fable/high".to_owned(),
                    took: String::new(),
                    state: "open: the window's own run".to_owned(),
                    cost: String::new(),
                    figures: Vec::new(),
                    result: None,
                    page: Some(id.to_owned()),
                },
                LedgerRow {
                    started: Some(Utc::now()),
                    started_at: String::new(),
                    short: "aaaaaaaa".to_owned(),
                    title: "meadow EP-2".to_owned(),
                    mode: "visual".to_owned(),
                    model: String::new(),
                    took: "52 min".to_owned(),
                    state: "completed".to_owned(),
                    cost: "$1.20".to_owned(),
                    figures: vec!["1 question, answered in 2 min".to_owned()],
                    result: Some("EP-2 is done.".to_owned()),
                    page: None,
                },
            ],
            problem: None,
        };
        let html = sessions(&page).unwrap();
        assert!(
            html.contains(r#"<main data-listen="/m/sessions""#),
            "{html}"
        );
        assert!(html.contains(&format!(
            r#"<a class="name" href="/m/s/{id}">harbor EP-4</a>"#
        )));
        assert_eq!(
            html.matches(&format!("href=\"/m/s/{id}\"")).count(),
            2,
            "{html}"
        );
        assert!(html.contains(r#"<span class="mark busy">running</span>"#));
        assert!(html.contains(r#"<span class="mark good">chained</span>"#));
        assert!(html.contains("A session is starting in the window"));
        assert!(html.contains("It waits for you in VS Code: a permission prompt."));
        assert!(
            html.contains("The &lt;berth&gt; list") || html.contains("The &#60;berth&#62; list"),
            "{html}"
        );
        assert!(html.contains("since "), "{html}");
        assert!(html.contains("2 more sessions run outside the registered projects."));
        assert!(html.contains("<strong>meadow EP-2</strong>"));
        assert!(html.contains("aaaaaaaa · 52 min · $1.20 · 1 question, answered in 2 min"));
        assert!(html.contains(r#"<span class="mark good">completed</span>"#));
        assert!(html.contains("EP-2 is done."));
        for part in ["sessions", "ledger"] {
            assert!(html.contains(&format!(r#"data-part="{part}""#)), "{part}");
        }
        // Nothing to list.
        page.sessions = SessionList::default();
        let html = sessions(&page).unwrap();
        assert!(html.contains("None: a headless run started from the window"));
        assert!(html.contains("No session in the ledger yet."));
    }

    #[test]
    fn the_notifications_and_the_doctors_report_draw_as_crw_reads_them() {
        use crate::published::{Doctor, LogLine, Notifications};
        let mut page = published(Vec::new());
        page.notifications = Notifications {
            read: true,
            rows: vec![LogLine {
                at: Some(Utc::now()),
                project: Some("harbor".to_owned()),
                words: "popup · waiting · harbor EP-4 waits for you <now>".to_owned(),
            }],
            problem: None,
        };
        let html = notifications(&page).unwrap();
        assert!(html.contains(r#"<main data-listen="/m/notifications""#));
        assert!(
            html.contains("<strong>harbor</strong> popup · waiting · harbor EP-4 waits for you")
                && (html.contains("you &lt;now&gt;") || html.contains("you &#60;now&#62;")),
            "{html}"
        );
        assert!(html.contains("The latest 20, the newest first"));
        page.notifications = Notifications::default();
        assert!(
            notifications(&page)
                .unwrap()
                .contains("The window keeps no ledger here")
        );
        // The doctor's report by group, each level in its class, with when it was gathered.
        page.doctor = Doctor {
            live: true,
            checking: false,
            report: Some((
                crate::diagnostics::canned(),
                Utc::now() - chrono::TimeDelta::minutes(3),
            )),
        };
        let html = diagnostics(&page, None).unwrap();
        assert!(html.contains(r#"<main data-listen="/m/diagnostics""#));
        assert!(html.contains("<h2>Tools</h2>") && html.contains("<h2>Backup drive</h2>"));
        assert!(
            html.contains(r#"<span class="level good">ok</span> <strong>claude</strong>"#),
            "{html}"
        );
        assert!(html.contains(r#"<span class="level bad">fail</span> <strong>Stop</strong>"#));
        assert!(
            html.contains(r#"<span class="level warn">warn</span> <strong>tailscale</strong>"#)
        );
        assert!(html.contains(", 3 min ago."), "{html}");
        // The real window's page offers Check again (the owner's choice), a form of no field.
        assert!(html.contains(r#"<form method="post" action="/m/diagnostics/check""#));
        assert!(html.contains("Check again</button>"));
        assert_eq!(html.matches("<form").count(), 1);
        // None gathered yet, and a check under way.
        page.doctor = Doctor {
            live: true,
            checking: true,
            report: None,
        };
        let html = diagnostics(&page, None).unwrap();
        assert!(html.contains("No report yet: the window gathers one when its Diagnostics pane"));
        assert!(html.contains("Checking now"));
        assert!(!html.contains("<form"), "no second check while one runs");
        // A window that gathers none (a screenshot run, a test) offers no Check again.
        page.doctor.live = false;
        page.doctor.checking = false;
        let html = diagnostics(&page, None).unwrap();
        assert!(html.contains("This window gathers no report") && !html.contains("<form"));
    }

    /// The script's append mode (EP-31): a numbered part's children appended, changed or let go,
    /// never set from text.
    #[test]
    fn the_script_appends_a_numbered_parts_children() {
        assert!(SCRIPT.contains(r#"part.hasAttribute("data-append")"#));
        assert!(SCRIPT.contains(r#"getAttribute("data-n")"#));
        assert!(SCRIPT.contains("appendChild(document.importNode(child, true))"));
        assert!(!SCRIPT.contains("innerHTML = ") && !SCRIPT.contains("insertAdjacentHTML"));
    }
}
