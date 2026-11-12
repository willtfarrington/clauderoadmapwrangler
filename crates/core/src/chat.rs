//! The chat's hosted session (D-27; EP-24): one read-only Claude Code session per conversation,
//! started through the runner like a headless run, each of the owner's messages a turn that
//! resumes the session by its id, with a digest of the portfolio appended to its system prompt.
//! The session's working folder is the tool's own `chat` folder under the local folder, and
//! every registered project that is not marked a data repository joins it read-only with
//! `--add-dir`, the plugin copy's folder too, so that the house-style reference reads;
//! `--restricted` confines the file tools to those folders, removes the shell and the web and
//! leaves the owner's settings and MCP servers out, and `--tools` names what is left. The run
//! starts in the CLI's `manual` mode, every permission request answered by the window with a
//! refusal and every question routed to the owner as any run's (`dontAsk` would deny the
//! questions too, Claude Code's permission-modes page, read 2026-11-11). The window builds the
//! digest from its own readings, never from a `crw status` over the registry (EP-7's note).

use std::path::{Path, PathBuf};

use crate::paths;
use crate::plugin;
use crate::session::{Appended, Launch, edit_deny_rule};

/// The project name a conversation's runs carry in the ledger: `crw sessions chat` lists them.
pub const PROJECT: &str = "chat";

/// The model and effort a conversation starts with until EP-24a's picker (D-27).
pub const MODEL: &str = "claude-opus-5-5";
pub const EFFORT: &str = "medium";

/// The folder under the local folder: the session's working folder and the digests' place.
pub const DIR: &str = "chat";

/// The built-in tools the session keeps (D-27): the readers, the plugin's skills and the
/// questions; no shell until EP-24b admits the tool's own commands.
pub const TOOLS: [&str; 5] = ["Read", "Glob", "Grep", "Skill", "AskUserQuestion"];

/// The permission mode the session starts in: the CLI's own default for a `-p` run.
pub const PERMISSION_MODE: &str = "manual";

/// The most of the digest, in characters: cut at a project's boundary beyond it.
pub const DIGEST_CAP: usize = 12_000;

/// The most of a conversation's title, from its first prompt.
pub const TITLE_CHARS: usize = 60;

/// What the window answers a chat run's permission request with (D-27): the session reads.
pub const REFUSAL: &str = "The chat session is read-only: it reads the folders it was given and writes nothing; \
     anything that writes into a project is the owner's, through the window.";

/// `%LOCALAPPDATA%\clauderoadmapwrangler\chat`, or the same under `CRW_HOME`; none without a
/// local folder.
pub fn dir() -> Option<PathBuf> {
    Some(paths::local_dir()?.join(DIR))
}

/// Where a conversation's digest is written for the record: `chat\digest-<id>.md`.
pub fn digest_file(dir: &Path, session_id: &str) -> PathBuf {
    dir.join(format!("digest-{session_id}.md"))
}

/// A conversation's title from its first prompt: its first line that holds anything, cut to
/// [`TITLE_CHARS`] with an ellipsis; "A conversation" for a blank prompt.
pub fn title_of(prompt: &str) -> String {
    let line = prompt
        .lines()
        .map(str::trim)
        .find(|line| !line.is_empty())
        .unwrap_or("A conversation");
    let words: String = line.split_whitespace().collect::<Vec<_>>().join(" ");
    if words.chars().count() > TITLE_CHARS {
        let cut: String = words.chars().take(TITLE_CHARS - 1).collect();
        format!("{}…", cut.trim_end())
    } else {
        words
    }
}

/// What the window knows of the portfolio at a turn's start, for the digest.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Digest {
    /// Today's date, `2026-11-11`.
    pub date: String,
    /// The projects the session reads, in the list's order.
    pub projects: Vec<ProjectDigest>,
    /// The projects it cannot read, each with why.
    pub left_out: Vec<(String, String)>,
    /// The account's usage in words, when the meter has a reading.
    pub usage: Vec<String>,
    /// The house-style reference in the plugin copy, when the copy is there.
    pub reference: Option<PathBuf>,
}

/// One project as the digest tells it.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ProjectDigest {
    pub name: String,
    pub state: String,
    pub folder: PathBuf,
    /// Rows done and rows in all; none without a roadmap the parser reads.
    pub counts: Option<(usize, usize)>,
    /// The next brief: its EP, size and title; none when every row is checked or no roadmap.
    pub next: Option<Next>,
    pub open_tasks: usize,
    pub open_messages: usize,
    pub urgent: usize,
    /// The sessions at work there, in words.
    pub sessions: Vec<String>,
    /// The last commit's date, `2026-11-10`.
    pub last_commit: Option<String>,
    pub hosting: Option<String>,
    /// This week's share, in words.
    pub usage: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Next {
    pub ep: String,
    pub size: Option<String>,
    pub title: String,
}

/// The text appended to the session's system prompt: the framing, the portfolio project by
/// project, what is left out, the house-style reference and the usage, kept to [`DIGEST_CAP`]
/// characters at a project's boundary.
pub fn digest_text(digest: &Digest) -> String {
    let mut parts: Vec<String> = vec![
        format!(
            "You are the chat of Roadmap Wrangler, the owner's dashboard over their Claude Code \
             projects, which plans each project as session-sized roadmap briefs. This session \
             reads the portfolio and answers in plain words; it writes nothing. Today is {}.",
            digest.date
        ),
        "What you can do: read the projects below, each by its folder, with Read, Glob and Grep, \
         and the house-style reference. The files are the truth; this digest is the window's \
         reading at the start of this turn. When the owner asks what a brief plans, read its row \
         and its brief note under the phase table of the project's roadmap/README.md and its \
         brief file (roadmap/EP-n-*.md), and say it in plain words, without the references and \
         pickup notes the notes carry for the sessions that run them. Owner tasks are in \
         roadmap/owner-tasks.md, inbox messages in roadmap/inbox.md, the decisions in \
         DECISIONS.md."
            .to_owned(),
        "What you cannot do: write a file, run a command or reach the network, and this session \
         is not given any other folder. Anything that writes into a project is the owner's, \
         through the window: Adopt (a folder of the data root), Formalise… and Migrate… (a \
         project's row menu), Compose… (a message into a project's inbox), Start (a session on \
         the next brief). Name the surface instead of trying."
            .to_owned(),
    ];
    if let Some(reference) = &digest.reference {
        parts.push(format!(
            "The house style every project follows (the phase tables, the briefs with their \
             completion notes, the decision log, owner tasks and the inbox) is described in {}; \
             the /roadmap-wrangler:status skill reads a project's standing the way the window \
             does.",
            reference.display()
        ));
    }
    let head = parts.join("\n\n");
    let mut text = head;
    text.push_str(&format!(
        "\n\nThe portfolio: {} readable, {} not.",
        count(digest.projects.len(), "project"),
        digest.left_out.len()
    ));
    // The tail first, so that the project lines leave it room under the cap.
    let mut tail = String::new();
    if !digest.left_out.is_empty() {
        let names: Vec<String> = digest
            .left_out
            .iter()
            .map(|(name, why)| format!("{name} ({why})"))
            .collect();
        tail.push_str(&format!(
            "\n\nNot readable by this session: {}.",
            names.join("; ")
        ));
    }
    if !digest.usage.is_empty() {
        tail.push_str(&format!(
            "\n\nThe account's usage: {}.",
            digest.usage.join("; ")
        ));
    }
    let cap = DIGEST_CAP.saturating_sub(tail.chars().count());
    // The project lines stop short of the cap by the room the rest's names take.
    const NAMES_ROOM: usize = 300;
    let mut shown = 0;
    for (index, project) in digest.projects.iter().enumerate() {
        let line = format!("\n\n- {}", project_line(project));
        let reserve = if index + 1 < digest.projects.len() {
            NAMES_ROOM
        } else {
            0
        };
        if text.chars().count() + line.chars().count() > cap.saturating_sub(reserve) {
            break;
        }
        text.push_str(&line);
        shown += 1;
    }
    let left = digest.projects.len() - shown;
    if left > 0 {
        // The names of the rest, as many as the room takes, so that the session can still read
        // their folders by name.
        let mut names = String::new();
        let room = cap.saturating_sub(text.chars().count() + 60);
        for project in &digest.projects[shown..] {
            let next = if names.is_empty() {
                project.name.clone()
            } else {
                format!("{names}, {}", project.name)
            };
            if next.chars().count() > room {
                names.push_str(if names.is_empty() { "…" } else { ", …" });
                break;
            }
            names = next;
        }
        text.push_str(&format!(
            "\n\n[and {} more, cut for room: {names}]",
            count(left, "project")
        ));
    }
    text.push_str(&tail);
    text
}

/// One project's line of the digest.
fn project_line(project: &ProjectDigest) -> String {
    let mut line = format!(
        "{} ({}): {}.",
        project.name,
        project.state,
        project.folder.display()
    );
    match (project.counts, &project.next) {
        (Some((done, total)), Some(next)) => {
            let size = next
                .size
                .as_deref()
                .map_or(String::new(), |size| format!(", {size}"));
            line.push_str(&format!(
                " Roadmap {done} of {total} rows done; next {}{size}: {}.",
                next.ep, next.title
            ));
        }
        (Some((done, total)), None) => {
            line.push_str(&format!(
                " Roadmap {done} of {total} rows done; every row checked."
            ));
        }
        (None, _) => line.push_str(" No roadmap the parser reads."),
    }
    let mut open = vec![count(project.open_tasks, "open owner task")];
    open.push(match project.urgent {
        0 => count(project.open_messages, "open inbox message"),
        n => format!(
            "{} ({n} urgent)",
            count(project.open_messages, "open inbox message")
        ),
    });
    line.push_str(&format!(" {}.", open.join(", ")));
    if !project.sessions.is_empty() {
        line.push_str(&format!(" Sessions: {}.", project.sessions.join("; ")));
    }
    if let Some(date) = &project.last_commit {
        line.push_str(&format!(" Last commit {date}."));
    }
    if let Some(hosting) = &project.hosting {
        line.push_str(&format!(" Hosting: {hosting}."));
    }
    if let Some(usage) = &project.usage {
        line.push_str(&format!(" Usage this week: {usage}."));
    }
    line
}

fn count(n: usize, noun: &str) -> String {
    match n {
        1 => format!("1 {noun}"),
        n => format!("{n} {noun}s"),
    }
}

/// Where a conversation's session runs and what it reads.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Places {
    /// The session's working folder: the tool's own [`DIR`].
    pub cwd: PathBuf,
    /// The folders joined read-only: each readable project's, and the plugin copy's.
    pub folders: Vec<PathBuf>,
    /// The digest, and where it was written.
    pub digest: Appended,
    /// The ledger the runs are recorded in; none for a trial.
    pub ledger: Option<PathBuf>,
}

/// The session a turn belongs to: a new one under `session_id`, chosen before the start so that
/// the digest's file is named by it, or the one resumed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Turn {
    First { session_id: String },
    Resume { session_id: String },
}

impl Turn {
    pub fn session_id(&self) -> &str {
        match self {
            Turn::First { session_id } | Turn::Resume { session_id } => session_id,
        }
    }

    /// A fresh id for a first turn.
    pub fn new() -> Self {
        Turn::First {
            session_id: uuid::Uuid::new_v4().to_string(),
        }
    }
}

impl Default for Turn {
    fn default() -> Self {
        Self::new()
    }
}

/// The launch of one turn of the conversation `title`: a new session, or the one `turn` resumes
/// continued with `prompt` as its next message (D-27; EP-24).
pub fn launch(title: &str, prompt: &str, turn: &Turn, places: Places) -> Launch {
    let mut launch = Launch::new(PROJECT, &places.cwd, MODEL, prompt);
    launch.effort = Some(EFFORT.to_owned());
    launch.permission_mode = PERMISSION_MODE.to_owned();
    launch.neighbours = places.folders;
    launch.neighbour_claude_md = false;
    launch.restricted = true;
    launch.tools = Some(TOOLS.iter().map(|tool| (*tool).to_owned()).collect());
    launch.appended = Some(places.digest);
    launch.fresh_system_prompt = true;
    launch.ledger = places.ledger;
    match turn {
        Turn::First { session_id } => launch.session_id = Some(session_id.clone()),
        Turn::Resume { session_id } => launch.resume = Some(session_id.clone()),
    }
    launch.conversation = Some(title.to_owned());
    launch
}

/// The folders a conversation reads: each registered project that is not marked a data
/// repository and whose folder is there, then the plugin copy's folder when it is there, so
/// that the house-style reference reads. `projects` gives each project's name, folder and
/// mark; the names left out come back with why.
pub fn folders<'a>(
    projects: impl IntoIterator<Item = (&'a str, Option<&'a Path>, bool)>,
    plugin_dir: Option<&Path>,
) -> (Vec<PathBuf>, Vec<(String, String)>) {
    let mut folders = Vec::new();
    let mut left_out = Vec::new();
    for (name, folder, data) in projects {
        match folder {
            _ if data => left_out.push((
                name.to_owned(),
                "a data repository, marked data = true in the registry".to_owned(),
            )),
            // A folder off a drive gets no deny rule, which would refuse every turn (the
            // review's finding): left out and said.
            Some(folder) if edit_deny_rule(folder).is_none() => left_out.push((
                name.to_owned(),
                "its folder is not on a drive, so no rule can keep it read-only".to_owned(),
            )),
            Some(folder) => folders.push(folder.to_owned()),
            None => left_out.push((name.to_owned(), "its folder is missing".to_owned())),
        }
    }
    if let Some(dir) = plugin_dir {
        folders.push(dir.to_owned());
    }
    (folders, left_out)
}

/// The house-style reference's path in the plugin copy, when the copy is there.
pub fn reference(plugin_dir: Option<&Path>) -> Option<PathBuf> {
    let file = plugin_dir?.join("reference").join("house-style.md");
    file.is_file().then_some(file)
}

/// The plugin copy's folder, as [`plugin::dir`] names it.
pub fn plugin_dir() -> Option<PathBuf> {
    plugin::dir()
}

#[cfg(test)]
mod tests {
    use std::ffi::OsString;
    use std::fs;

    use super::*;
    use crate::files::tests::Scratch;
    use crate::session::cli::Flags;

    const HELP: &str = "Options:
  --add-dir <directories...>  Additional directories
  --append-system-prompt <prompt>  Append a system prompt
  --effort <level>  Effort level (low, medium, high, xhigh, max)
  --include-partial-messages  Partial chunks
  --input-format <format>  Input format
  --model <model>  Model
  -n, --name <name>  Display name
  --output-format <format>  Output format
  --permission-mode <mode>  Mode (choices: \"acceptEdits\", \"auto\", \"manual\", \"dontAsk\")
  --permission-prompts <target>  Who answers permission prompts with
                                 --print: \"host\" (the SDK host or
                                 --permission-prompt-tool) or \"none\"
  --plugin-dir <path>  Load a plugin
  -p, --print  Print
  --replay-user-messages  Echo
  --restricted  Restricted mode
  -r, --resume [value]  Resume
  --session-id <uuid>  Session id
  --settings <file-or-json>  Settings
  --strict-mcp-config  Only --mcp-config
  --system-prompt-snapshot <on|off>  Record the system prompt
  --tools <tools...>  Specify the list of available tools
  --verbose  Verbose
";

    fn one(name: &str) -> ProjectDigest {
        ProjectDigest {
            name: name.to_owned(),
            state: "active".to_owned(),
            folder: PathBuf::from(format!("C:\\Data\\{name}")),
            counts: Some((3, 8)),
            next: Some(Next {
                ep: "EP-4".to_owned(),
                size: Some("M".to_owned()),
                title: "Ledger view".to_owned(),
            }),
            open_tasks: 1,
            open_messages: 2,
            urgent: 1,
            sessions: vec!["a headless EP-4 started here at 10:02 is running".to_owned()],
            last_commit: Some("2026-11-10".to_owned()),
            hosting: Some("private".to_owned()),
            usage: Some("week +2 % · Fable +1 %".to_owned()),
        }
    }

    #[test]
    fn the_digest_says_the_portfolio_in_plain_lines() {
        let digest = Digest {
            date: "2026-11-11".to_owned(),
            projects: vec![
                one("harbor"),
                ProjectDigest {
                    name: "lantern".to_owned(),
                    state: "paused".to_owned(),
                    folder: PathBuf::from("C:\\Data\\lantern"),
                    counts: Some((5, 5)),
                    ..ProjectDigest::default()
                },
                ProjectDigest {
                    name: "tundra".to_owned(),
                    state: "unformalised".to_owned(),
                    folder: PathBuf::from("C:\\Data\\tundra"),
                    ..ProjectDigest::default()
                },
            ],
            left_out: vec![(
                "quarry".to_owned(),
                "a data repository, marked data = true in the registry".to_owned(),
            )],
            usage: vec!["weekly 31 % of the limit".to_owned()],
            reference: Some(PathBuf::from(
                "C:\\Local\\plugin\\reference\\house-style.md",
            )),
        };
        let text = digest_text(&digest);
        assert!(text.starts_with("You are the chat of Roadmap Wrangler"));
        assert!(text.contains("Today is 2026-11-11."));
        assert!(text.contains("house-style.md; the /roadmap-wrangler:status skill"));
        assert!(text.contains("The portfolio: 3 projects readable, 1 not."));
        assert!(text.contains(
            "- harbor (active): C:\\Data\\harbor. Roadmap 3 of 8 rows done; next EP-4, M: \
             Ledger view. 1 open owner task, 2 open inbox messages (1 urgent). Sessions: a \
             headless EP-4 started here at 10:02 is running. Last commit 2026-11-10. Hosting: \
             private. Usage this week: week +2 % · Fable +1 %."
        ));
        assert!(text.contains(
            "- lantern (paused): C:\\Data\\lantern. Roadmap 5 of 5 rows done; every row \
             checked. 0 open owner tasks, 0 open inbox messages."
        ));
        assert!(
            text.contains(
                "- tundra (unformalised): C:\\Data\\tundra. No roadmap the parser reads."
            )
        );
        assert!(text.contains(
            "Not readable by this session: quarry (a data repository, marked data = true in \
             the registry)."
        ));
        assert!(text.ends_with("The account's usage: weekly 31 % of the limit."));
        assert!(!text.contains("[and"));
    }

    #[test]
    fn a_long_digest_is_cut_at_a_projects_boundary() {
        let digest = Digest {
            date: "2026-11-11".to_owned(),
            projects: (0..80).map(|n| one(&format!("project-{n:02}"))).collect(),
            left_out: vec![("quarry".to_owned(), "a data repository".to_owned())],
            usage: vec!["weekly 31 % of the limit".to_owned()],
            ..Digest::default()
        };
        let text = digest_text(&digest);
        assert!(
            text.chars().count() <= DIGEST_CAP,
            "{}",
            text.chars().count()
        );
        assert!(text.ends_with("The account's usage: weekly 31 % of the limit."));
        assert!(text.contains("Not readable by this session: quarry (a data repository)."));
        assert!(text.contains("- project-00 (active)"));
        assert!(text.contains("more, cut for room: project-"));
        assert!(text.contains("]\n\nNot readable"));
        // Every line shown is whole.
        for line in text.lines().filter(|line| line.starts_with("- ")) {
            assert!(line.ends_with('.'), "{line}");
        }
    }

    #[test]
    fn a_title_comes_from_the_first_line() {
        assert_eq!(
            title_of("\n\n  What does   EP-24 plan?\nMore."),
            "What does EP-24 plan?"
        );
        assert_eq!(title_of("   "), "A conversation");
        let long = title_of(&"word ".repeat(30));
        assert_eq!(long.chars().count(), TITLE_CHARS);
        assert!(long.ends_with('…'));
    }

    #[test]
    fn the_folders_leave_out_the_data_repositories() {
        let scratch = Scratch::new("chat-folders");
        let alpha = scratch.0.join("alpha");
        let plugin = scratch.0.join("plugin");
        fs::create_dir_all(&alpha).unwrap();
        fs::create_dir_all(&plugin).unwrap();
        let beta = scratch.0.join("beta");
        let (folders, left_out) = folders(
            [
                ("alpha", Some(alpha.as_path()), false),
                ("beta", Some(beta.as_path()), true),
                ("gamma", None, false),
                ("delta", Some(Path::new(r"\\server\share\delta")), false),
            ],
            Some(&plugin),
        );
        assert_eq!(folders, [alpha, plugin.clone()]);
        assert_eq!(left_out.len(), 3);
        assert_eq!(left_out[0].0, "beta");
        assert!(left_out[0].1.contains("data = true"));
        assert_eq!(
            left_out[1],
            ("gamma".to_owned(), "its folder is missing".to_owned())
        );
        assert_eq!(left_out[2].0, "delta");
        assert!(left_out[2].1.contains("not on a drive"));
        assert_eq!(reference(Some(&plugin)), None);
        let file = plugin.join("reference").join("house-style.md");
        fs::create_dir_all(file.parent().unwrap()).unwrap();
        fs::write(&file, "# House style").unwrap();
        assert_eq!(reference(Some(&plugin)), Some(file));
    }

    /// The command line of a turn (EP-24): the boundary flags, the tools, the digest inline, the
    /// session named after the conversation, the neighbours' `CLAUDE.md` files left unloaded,
    /// the resume; and the refusals of a claude without the boundary.
    #[test]
    fn a_turns_command_line_keeps_the_session_to_its_folders() {
        let scratch = Scratch::new("chat-launch");
        let cwd = scratch.0.join("chat");
        let alpha = scratch.0.join("alpha");
        let plugin = scratch.0.join("plugin");
        for folder in [&cwd, &alpha, &plugin] {
            fs::create_dir_all(folder).unwrap();
        }
        let digest = Appended {
            file: digest_file(&cwd, "0e1d"),
            text: "You are the chat.".to_owned(),
        };
        let places = Places {
            cwd: cwd.clone(),
            folders: vec![alpha.clone(), plugin.clone()],
            digest,
            ledger: None,
        };
        let first = Turn::First {
            session_id: "0e1d2c3b-4a59-4687-9a1b-2c3d4e5f6a7b".to_owned(),
        };
        let launch = launch(
            "What does EP-24 plan?",
            "What does EP-24 plan?",
            &first,
            places,
        );
        assert_eq!(launch.name(), "chat: What does EP-24 plan?");
        assert_eq!(launch.project, PROJECT);
        assert_eq!(launch.folder, cwd);
        assert_eq!(launch.session_id.as_deref(), Some(first.session_id()));
        assert!(launch.resume.is_none());
        let flags = Flags::parse(HELP);
        let line = crate::session::launch::command_line(
            &launch,
            &flags,
            "0e1d2c3b-4a59-4687-9a1b-2c3d4e5f6a7b",
            [OsString::from("PATH")],
        )
        .unwrap();
        let args: Vec<String> = line
            .args
            .iter()
            .map(|arg| arg.to_string_lossy().into_owned())
            .collect();
        let after = |flag: &str| -> Vec<&str> {
            args.windows(2)
                .filter(|pair| pair[0] == flag)
                .map(|pair| pair[1].as_str())
                .collect()
        };
        assert!(args.iter().any(|arg| arg == "--restricted"));
        assert!(args.iter().any(|arg| arg == "--strict-mcp-config"));
        assert_eq!(after("--tools"), ["Read,Glob,Grep,Skill,AskUserQuestion"]);
        assert_eq!(after("--permission-mode"), ["manual"]);
        assert_eq!(after("--permission-prompts"), ["host"]);
        assert_eq!(after("--permission-prompt-tool"), ["stdio"]);
        assert_eq!(after("--model"), [MODEL]);
        assert_eq!(after("--effort"), [EFFORT]);
        assert_eq!(after("--name"), ["chat: What does EP-24 plan?"]);
        assert_eq!(after("--append-system-prompt"), ["You are the chat."]);
        assert_eq!(after("--system-prompt-snapshot"), ["off"]);
        assert_eq!(
            after("--add-dir"),
            [alpha.display().to_string(), plugin.display().to_string()]
        );
        assert!(!args.iter().any(|arg| arg == "--bare"));
        assert!(line.dropped.is_empty(), "{:?}", line.dropped);
        // The neighbours' CLAUDE.md files stay unloaded: the key alone leaves the environment.
        assert_eq!(line.env, [(OsString::from("ANTHROPIC_API_KEY"), None)]);

        // A later turn resumes the session with the owner's message.
        let places = Places {
            cwd: cwd.clone(),
            folders: vec![alpha.clone()],
            digest: Appended {
                file: digest_file(&cwd, "0e1d"),
                text: "A fresh digest.".to_owned(),
            },
            ledger: None,
        };
        let resumed = Turn::Resume {
            session_id: "0e1d".to_owned(),
        };
        let turn = super::launch("What does EP-24 plan?", "And EP-24a?", &resumed, places);
        assert!(turn.session_id.is_none());
        let line = crate::session::launch::command_line(&turn, &flags, "0e1d", []).unwrap();
        let args: Vec<String> = line
            .args
            .iter()
            .map(|arg| arg.to_string_lossy().into_owned())
            .collect();
        assert!(
            args.windows(2)
                .any(|pair| pair[0] == "--resume" && pair[1] == "0e1d")
        );
        assert!(args.contains(&"A fresh digest.".to_owned()));

        // The digest by file when the help lists the flag and the file is there; inline until
        // the file is written.
        let by_file = Flags::parse(&format!(
            "{HELP}  --append-system-prompt-file <file>  Append a file\n"
        ));
        let line = crate::session::launch::command_line(&turn, &by_file, "0e1d", []).unwrap();
        assert!(line.args.iter().any(|arg| arg == "A fresh digest."));
        fs::write(digest_file(&cwd, "0e1d"), "A fresh digest.").unwrap();
        let line = crate::session::launch::command_line(&turn, &by_file, "0e1d", []).unwrap();
        let args: Vec<String> = line
            .args
            .iter()
            .map(|arg| arg.to_string_lossy().into_owned())
            .collect();
        assert!(
            args.windows(2)
                .any(|pair| pair[0] == "--append-system-prompt-file"
                    && pair[1] == digest_file(&cwd, "0e1d").display().to_string())
        );
        assert!(!args.iter().any(|arg| arg == "--append-system-prompt"));

        // Without the boundary, no chat: each missing flag refuses the launch.
        for flag in ["--restricted", "--strict-mcp-config", "--tools"] {
            let without = Flags::parse(&HELP.replace(flag, "--other"));
            let why =
                crate::session::launch::command_line(&turn, &without, "0e1d", []).unwrap_err();
            assert!(why.contains(flag), "{why}");
        }
        // A claude that lists no manual mode starts the run in its default mode, and says so; one
        // without the snapshot flag or the appended prompt goes without, said too.
        let older = Flags::parse(
            &HELP
                .replace(", \"manual\"", "")
                .replace("--system-prompt-snapshot", "--other")
                .replace("--append-system-prompt <prompt>", "--other-prompt <p>"),
        );
        let line = crate::session::launch::command_line(&turn, &older, "0e1d", []).unwrap();
        let args: Vec<String> = line
            .args
            .iter()
            .map(|arg| arg.to_string_lossy().into_owned())
            .collect();
        assert!(!args.iter().any(|arg| arg == "--permission-mode"));
        assert!(!args.iter().any(|arg| arg == "--system-prompt-snapshot"));
        assert_eq!(line.dropped.len(), 3, "{:?}", line.dropped);
        assert!(line.dropped[0].contains("manual"));
        assert!(line.dropped[1].contains("the digest"));
        assert!(line.dropped[2].contains("--system-prompt-snapshot"));
    }
}
