//! The phone's pages (D-14, D-28; EP-30): askama templates in the crate's `templates` folder,
//! rendered from what the window publishes (`published`), and the stylesheet they share, served
//! from the exe. A page loads nothing from outside, works with no script, and follows the phone's
//! own light or dark setting. EP-30's page is the project list, which reads only; EP-30a adds a
//! project's page (its next brief's note, what waits, the inbox, a message to it) and the
//! decisions, whose forms post to the window and come back with one line saying what happened;
//! EP-30b adds the actions and the live updates.

use askama::Template;
use chrono::{DateTime, Local, Utc};
use crw_core::export::escape;
use crw_core::registry::State;

use crate::form::segment;
use crate::markup::{self, Block};
use crate::published::{Asking, How, Item, Line, Next, Published};
use crate::server::Said;

/// The stylesheet every page links, at `/m/app.css`.
pub const STYLESHEET: &str = include_str!("../templates/app.css");

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

impl LineView {
    fn of(line: &Line) -> Self {
        let mut marks = Vec::new();
        if line.pinned {
            marks.push(("pinned".to_owned(), "weak"));
        }
        if line.proposes {
            marks.push(("state change proposed".to_owned(), "warn"));
        }
        if line.autopilot {
            marks.push(("autopilot on".to_owned(), "good"));
        }
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
            1 => waits.push(("1 owner item".to_owned(), "warn")),
            n => waits.push((format!("{n} owner items"), "warn")),
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
    summary: String,
    /// What waits for an answer, for the link to the decisions page (EP-30a).
    decisions: String,
    problem: Option<String>,
    reading: bool,
    groups: Vec<GroupView>,
    unadopted: Vec<String>,
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

#[derive(Template)]
#[template(path = "project.html")]
struct ProjectPage {
    as_of: String,
    said: Option<SaidView>,
    name: String,
    link: String,
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
}

impl AskView {
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
    let local = at.with_timezone(&Local);
    if local.date_naive() == Local::now().date_naive() {
        local.format("%H:%M").to_string()
    } else {
        local.format("%a %d %b %H:%M").to_string()
    }
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
    // The marks the list's line shows after the state (the review's: the brief names them).
    let marks = published
        .list
        .projects()
        .find(|line| line.name.eq_ignore_ascii_case(&project.name))
        .map(|line| LineView::of(line).marks)
        .unwrap_or_default();
    Some(
        ProjectPage {
            as_of: as_of(published.at),
            said: SaidView::of(said),
            name: project.name.clone(),
            link: project_link(&project.name),
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

/// The list page of `published`.
pub fn list(published: &Published) -> Result<String, askama::Error> {
    let list = &published.list;
    let projects: Vec<&Line> = list.projects().collect();
    let owner_items: usize = projects.iter().map(|line| line.owner_items).sum();
    let messages: usize = projects.iter().map(|line| line.messages).sum();
    let summary = format!(
        "{} · {} · {}",
        plural(projects.len(), "project"),
        plural(owner_items, "owner item"),
        plural(messages, "inbox message")
    );
    let decisions = match published.decisions.count() {
        0 => "Nothing waits for an answer".to_owned(),
        1 => "1 waits for an answer".to_owned(),
        n => format!("{n} wait for an answer"),
    };
    ListPage {
        as_of: as_of(published.at),
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
        Published {
            at: Utc::now(),
            version: 3,
            list: List {
                reading: false,
                problem: None,
                groups,
                unadopted: vec!["new-folder".to_owned()],
            },
            projects: Vec::new(),
            decisions: Decisions::default(),
        }
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
        // Nothing from outside, no script.
        assert!(!html.contains("http://") && !html.contains("https://"));
        assert!(!html.contains("<script"));
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
        // The list line's marks beside the state.
        let mut pinned = line(
            "harbor & co",
            State::Active,
            Next::None {
                words: "Every row is checked".to_owned(),
                missing: false,
            },
        );
        pinned.pinned = true;
        pinned.autopilot = true;
        page.list.groups = vec![Group {
            state: State::Active,
            projects: vec![pinned],
        }];
        page.projects = vec![harbor_page()];
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
        let html = list(&page).unwrap();
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
        }];
        let done = line(
            "orchard",
            State::Done,
            Next::None {
                words: "Every row is checked".to_owned(),
                missing: false,
            },
        );
        let page = list(&published(vec![
            Group {
                state: State::Active,
                projects: vec![harbor],
            },
            Group {
                state: State::Done,
                projects: vec![done],
            },
        ]))
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
        assert!(page.contains("2 owner items"));
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
        assert!(page.contains("2 projects · 2 owner items · 3 inbox messages"));
        // Nothing from outside the window's own server.
        assert!(
            !page.contains("http://") && !page.contains("https://"),
            "{page}"
        );
        assert!(page.contains(r#"href="/m/app.css""#));
        assert!(!page.contains("<script"));
    }

    #[test]
    fn a_list_not_read_yet_says_so_without_a_time() {
        let mut empty = published(Vec::new());
        empty.at = DateTime::<Utc>::MIN_UTC;
        empty.list.reading = true;
        empty.list.unadopted.clear();
        let page = list(&empty).unwrap();
        assert!(page.contains("Reading the projects…"));
        assert!(!page.contains("as of"));
        assert!(page.contains("0 projects"));
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
}
