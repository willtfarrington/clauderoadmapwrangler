//! The phone's pages (D-14, D-28; EP-30): askama templates in the crate's `templates` folder,
//! rendered from what the window publishes (`published`), and the stylesheet they share, served
//! from the exe. A page loads nothing from outside, works with no script, and follows the phone's
//! own light or dark setting. This brief's page is the project list, which reads only; EP-30a adds
//! a project's page and the decisions, EP-30b the actions and the live updates.

use askama::Template;
use chrono::{DateTime, Local, Utc};
use crw_core::registry::State;

use crate::published::{Line, Next, Published};

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
    problem: Option<String>,
    reading: bool,
    groups: Vec<GroupView>,
    unadopted: Vec<String>,
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
    ListPage {
        as_of: as_of(published.at),
        summary,
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
    use crate::published::{Badge, Group, List};
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
        }
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
