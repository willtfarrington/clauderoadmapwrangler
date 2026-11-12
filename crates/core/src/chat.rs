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

/// The model and effort a new conversation starts with (D-27), until its picker says otherwise
/// (EP-24a): the choice is the conversation's and goes with each of its turns.
pub const MODEL: &str = "claude-opus-5-5";
pub const EFFORT: &str = "medium";

/// The folder under the local folder: the session's working folder and the digests' place.
pub const DIR: &str = "chat";

/// The built-in tools the session keeps (D-27): the readers, the plugin's skills, the questions
/// and, from EP-24b, the shell for the tool's own `crw` reading commands, which the rules of
/// [`allowed_rules`] let through without a prompt; any other command's request is refused by
/// the window.
pub const TOOLS: [&str; 6] = ["Read", "Glob", "Grep", "Skill", "AskUserQuestion", "Bash"];

/// The name the rules and the digest call the tool's CLI by: the child's `PATH` starts with the
/// folder the window runs beside, so the bare name is found (never a path, which the owner's
/// user name would be in).
pub const CLI: &str = "crw";

/// The `crw` commands the session may run on one readable project, each an exact rule per
/// project (`Bash(crw roadmap alpha)`), so that a data repository's name, a path or a command
/// with a pipe never matches a rule.
pub const PROJECT_COMMANDS: [&str; 3] = ["roadmap", "tasks", "sessions"];

/// The ones it may run bare, with no argument.
pub const BARE_COMMANDS: [&str; 5] = ["sessions", "mail", "notifications", "usage", "autopilot"];

/// The writing commands the session may run itself, each a prefix rule (`Bash(crw send:*)`):
/// none, on the owner's answer at EP-24b's pickup (2026-11-11), every write staying a chip for
/// the owner's click ([`Proposed`]; D-27, D-11).
pub const WRITING_COMMANDS: &[&str] = &[];

/// A project's name as a command line takes it: in double quotes when it holds a space, so
/// that the shell passes it as one argument and the rule matches the command as typed (the
/// review's finding); the digest spells the names the same way.
pub fn spelled(name: &str) -> String {
    if name.chars().any(char::is_whitespace) {
        format!("\"{name}\"")
    } else {
        name.to_owned()
    }
}

/// The `--allowedTools` rules of a conversation's turn: the project commands on each readable
/// project by name, the bare ones, and the writing ones the owner admits.
pub fn allowed_rules(readable: &[String]) -> Vec<String> {
    let mut rules = Vec::new();
    for name in readable {
        let name = spelled(name);
        for command in PROJECT_COMMANDS {
            rules.push(format!("Bash({CLI} {command} {name})"));
        }
    }
    for command in BARE_COMMANDS {
        rules.push(format!("Bash({CLI} {command})"));
    }
    for command in WRITING_COMMANDS {
        rules.push(format!("Bash({CLI} {command}:*)"));
    }
    rules
}

/// The fence's info string a reply tags its proposed actions with.
pub const ACTIONS_TAG: &str = "actions";

/// An action a reply proposes (D-27; EP-24b), read out of its `actions` block: the tile draws
/// each as a chip that opens the window's own dialog with the project filled in, and the dialog
/// writes only on the owner's click there.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Proposed {
    /// A folder of the data root not yet registered (D-7); the chip offers the data mark first.
    Adopt { folder: String },
    /// The formalise preview on a registered project (EP-21a).
    Formalise { project: String },
    /// The launch dialog with the migration prompt (EP-21a).
    Migrate { project: String },
    /// The compose dialog on a project's inbox, the words typed in (EP-20c).
    Compose { project: String, words: String },
    /// A line the window does not know: drawn as such, never taken.
    Unknown { line: String },
}

/// The most of a chip's label, in characters: a long unknown line would widen the pane past
/// its edge (EP-13's trap; the review's finding), so it is cut, the whole line in the hover.
pub const LABEL_CHARS: usize = 40;

impl Proposed {
    /// The chip's words: `Formalise alpha…`; an unknown line cut to [`LABEL_CHARS`].
    pub fn label(&self) -> String {
        match self {
            Proposed::Adopt { folder } => format!("Adopt {folder}…"),
            Proposed::Formalise { project } => format!("Formalise {project}…"),
            Proposed::Migrate { project } => format!("Migrate {project}…"),
            Proposed::Compose { project, .. } => format!("Compose to {project}…"),
            Proposed::Unknown { line } if line.chars().count() > LABEL_CHARS => {
                let cut: String = line.chars().take(LABEL_CHARS - 1).collect();
                format!("{}…", cut.trim_end())
            }
            Proposed::Unknown { line } => line.clone(),
        }
    }

    /// The project or folder the action names; none for an unknown line.
    pub fn target(&self) -> Option<&str> {
        match self {
            Proposed::Adopt { folder } => Some(folder),
            Proposed::Formalise { project }
            | Proposed::Migrate { project }
            | Proposed::Compose { project, .. } => Some(project),
            Proposed::Unknown { .. } => None,
        }
    }

    /// One line of the block, as the digest asks for it; a list marker before the verb (`- `,
    /// `* `, `1. `) is let go.
    fn parse(line: &str) -> Option<Self> {
        let line = line.trim();
        let line = ["- ", "* ", "+ "]
            .into_iter()
            .find_map(|marker| line.strip_prefix(marker))
            .or_else(|| {
                let digits = line.trim_start_matches(|c: char| c.is_ascii_digit());
                (digits.len() < line.len())
                    .then(|| {
                        digits
                            .strip_prefix(". ")
                            .or_else(|| digits.strip_prefix(") "))
                    })
                    .flatten()
            })
            .unwrap_or(line)
            .trim();
        if line.is_empty() {
            return None;
        }
        let (verb, rest) = line.split_once(char::is_whitespace).unwrap_or((line, ""));
        let name = |text: &str| text.trim().trim_start_matches('@').to_owned();
        Some(match verb.to_ascii_lowercase().as_str() {
            "adopt" if !rest.trim().is_empty() => Proposed::Adopt { folder: name(rest) },
            "formalise" | "formalize" if !rest.trim().is_empty() => Proposed::Formalise {
                project: name(rest),
            },
            "migrate" if !rest.trim().is_empty() => Proposed::Migrate {
                project: name(rest),
            },
            "compose" if !rest.trim().is_empty() => {
                let (project, words) = match rest.split_once(':') {
                    Some((project, words)) => (project, words),
                    None => rest.split_once(char::is_whitespace).unwrap_or((rest, "")),
                };
                Proposed::Compose {
                    project: name(project),
                    words: words.trim().to_owned(),
                }
            }
            _ => Proposed::Unknown {
                line: line.to_owned(),
            },
        })
    }
}

/// A fence line: its character (a backtick or a tilde), the run's length (three or more) and
/// what follows the run, trimmed; `None` for any other line.
fn fence_of(trimmed: &str) -> Option<(char, usize, &str)> {
    let first = trimmed.chars().next()?;
    if first != '`' && first != '~' {
        return None;
    }
    let run = trimmed.chars().take_while(|&c| c == first).count();
    if run < 3 {
        return None;
    }
    Some((first, run, trimmed[run..].trim()))
}

/// The actions a reply proposes and the reply without their block: every fenced block tagged
/// [`ACTIONS_TAG`] (a run of three or more backticks or tildes, the tag as its info string,
/// closed by a run of the same character at least as long with nothing after it, as
/// CommonMark closes a fence), one action per line, wherever it stands; a block the reply left
/// open at its end counts whole. The other fences are left as they are, a longer run inside
/// them included (the review's finding).
pub fn proposals(text: &str) -> (String, Vec<Proposed>) {
    let mut kept: Vec<&str> = Vec::new();
    let mut proposed = Vec::new();
    // The fence open now: its character, its run and whether it is an actions block.
    let mut fence: Option<(char, usize, bool)> = None;
    for line in text.lines() {
        let trimmed = line.trim_start();
        match fence {
            Some((open, run, actions)) => {
                let closes = fence_of(trimmed)
                    .is_some_and(|(c, n, rest)| c == open && n >= run && rest.is_empty());
                if closes {
                    fence = None;
                    if !actions {
                        kept.push(line);
                    }
                } else if actions {
                    proposed.extend(Proposed::parse(line));
                } else {
                    kept.push(line);
                }
            }
            None => match fence_of(trimmed) {
                Some((open, run, info)) => {
                    let actions = info.eq_ignore_ascii_case(ACTIONS_TAG);
                    fence = Some((open, run, actions));
                    if !actions {
                        kept.push(line);
                    }
                }
                None => kept.push(line),
            },
        }
    }
    (kept.join("\n").trim_end().to_owned(), proposed)
}

/// The permission mode the session starts in: the CLI's own default for a `-p` run.
pub const PERMISSION_MODE: &str = "manual";

/// The most of the digest, in characters: cut at a project's boundary beyond it.
pub const DIGEST_CAP: usize = 12_000;

/// The most of a conversation's title, from its first prompt.
pub const TITLE_CHARS: usize = 60;

/// What the window answers a chat run's permission request with (D-27): the session reads, and
/// runs only the commands the rules admit, as the digest spells them.
pub const REFUSAL: &str = "The chat session is read-only: it reads the folders it was given, runs only the crw \
     reading commands the digest lists, each exactly as listed and alone on its line, and writes \
     nothing; anything that writes into a project is the owner's, through the window: offer it \
     as an action in an actions block.";

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
    /// The folders of the data root not yet adopted nor ignored (D-7), for an Adopt action.
    pub unadopted: Vec<String>,
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
    /// The project keeps no `roadmap/owner-tasks.md`, so `open_tasks` counts the items of the
    /// older forms (a README's risks list among them), as `crw tasks` says (IN-12, EP-25).
    pub older_forms: bool,
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
        commands_text(),
        format!(
            "What you cannot do: write a file, run any other command or reach the network, and \
             this session is not given any other folder. Anything that writes into a project is \
             the owner's, through the window, and you offer it as an action: end the reply with \
             a fenced block tagged {ACTIONS_TAG}, one action per line, in these forms:\n\
             ```{ACTIONS_TAG}\n\
             adopt <folder>\n\
             formalise <project>\n\
             migrate <project>\n\
             compose <project>: <the message's words>\n\
             ```\n\
             The window draws each as a button under the reply, which opens its own dialog with \
             the project filled in (Adopt registers a folder of the data root, asking first \
             whether it is a data repository; Formalise previews the house-style files for a \
             registered project; Migrate opens a session that converts an older roadmap to the \
             house tables; Compose opens a message into the project's inbox with the words \
             typed, creating roadmap/inbox.md when the project has none, so a project need not \
             be formalised first to be written to), and nothing is written until the owner \
             clicks there. Offer an action only \
             when the owner asks for it or it is the plain next step of what they asked, and \
             say in the reply what it does. Starting a session on a project's next brief stays \
             the window's row: name it instead."
        ),
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
    if !digest.unadopted.is_empty() {
        tail.push_str(&format!(
            "\n\nFolders of the data root not yet adopted (an Adopt action registers one): {}.",
            digest.unadopted.join(", ")
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

/// The digest's paragraph on the commands the session may run (EP-24b), spelled as the rules
/// match them.
fn commands_text() -> String {
    let project: Vec<String> = PROJECT_COMMANDS
        .iter()
        .map(|command| format!("`{CLI} {command} <project>`"))
        .collect();
    let bare: Vec<String> = BARE_COMMANDS
        .iter()
        .map(|command| format!("`{CLI} {command}`"))
        .collect();
    let mut text = format!(
        "What you can run, with Bash, each exactly as written here and alone on its line (no \
         pipe, no redirect, no other command): {} with <project> the name of one readable \
         project below (in double quotes when it holds a space), and {}. {CLI} roadmap prints \
         a project's roadmap as the window reads it \
         (the phases, each row's state, the next brief), {CLI} tasks its open owner tasks and \
         inbox messages, {CLI} sessions the ledger's sessions, {CLI} mail the deliveries between \
         projects, {CLI} notifications the window's latest, {CLI} usage the account's figures, \
         {CLI} autopilot the chains' state. Any other command is refused by the window.",
        project.join(", "),
        bare.join(", ")
    );
    if !WRITING_COMMANDS.is_empty() {
        let writing: Vec<String> = WRITING_COMMANDS
            .iter()
            .map(|command| format!("`{CLI} {command} …`"))
            .collect();
        text.push_str(&format!(
            " You may also run {}, which write, only when the owner asked for exactly that.",
            writing.join(", ")
        ));
    }
    text
}

/// The text appended to the session's system prompt for one turn (EP-24b): the digest, then
/// the context of the projects the owner's message mentions with `@`, when it names any (the
/// prompt hook's own, `signals::mentions::context`, which leaves an SDK's session out).
pub fn appended(digest: &str, mentions: Option<&str>) -> String {
    match mentions {
        Some(context) => format!(
            "{digest}\n\nThe owner's message names a project with @. Its context, as the \
             prompt hook gives an interactive session (the next row and the start of its brief \
             note, its rules in short, then the paths of the rest); where it says a plan \
             changes through a message in the project's inbox, that message is a compose \
             action of yours, never a write:\n\n{context}"
        ),
        None => digest.to_owned(),
    }
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
    let mut open = vec![if project.older_forms {
        format!(
            "{} (no owner-tasks.md: items in older forms, a risks list among them, as crw \
             tasks lists them)",
            count(project.open_tasks, "open item")
        )
    } else {
        count(project.open_tasks, "open owner task")
    }];
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
    /// The readable projects' names, for the rules of what the session may run (EP-24b).
    pub readable: Vec<String>,
    /// The folder the window runs beside, where `crw.exe` is: the child's `PATH` starts with
    /// it so that the rules' bare `crw` is found; none for a build with no `crw.exe` beside it,
    /// whose session's commands then fail as not found.
    pub bin: Option<PathBuf>,
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

/// The model and effort a turn runs with: the conversation's picker (EP-24a), or the defaults.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pick {
    /// A model alias or id, passed as written.
    pub model: String,
    /// None for the CLI's default effort.
    pub effort: Option<String>,
}

impl Default for Pick {
    fn default() -> Self {
        Self {
            model: MODEL.to_owned(),
            effort: Some(EFFORT.to_owned()),
        }
    }
}

impl Pick {
    /// `claude-opus-5-5 at medium`, or `… at the CLI's default effort`.
    pub fn words(&self) -> String {
        match &self.effort {
            Some(effort) => format!("{} at {effort}", self.model),
            None => format!("{} at the CLI's default effort", self.model),
        }
    }
}

/// The launch of one turn of the conversation `title`: a new session, or the one `turn` resumes
/// continued with `prompt` as its next message (D-27; EP-24), run with `pick`'s model and effort.
pub fn launch(title: &str, prompt: &str, turn: &Turn, places: Places, pick: &Pick) -> Launch {
    let mut launch = Launch::new(PROJECT, &places.cwd, &pick.model, prompt);
    launch.effort = pick.effort.clone();
    launch.permission_mode = PERMISSION_MODE.to_owned();
    launch.neighbours = places.folders;
    launch.neighbour_claude_md = false;
    launch.restricted = true;
    launch.tools = Some(TOOLS.iter().map(|tool| (*tool).to_owned()).collect());
    launch.allowed_tools = allowed_rules(&places.readable);
    launch.path_prepend = places.bin;
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
  --allowedTools, --allowed-tools <tools...>
      Comma or space-separated list of tool names to allow
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
            older_forms: false,
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
                    // No owner-tasks.md: its risks list counts as items in older forms (IN-12).
                    open_tasks: 3,
                    older_forms: true,
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
            unadopted: vec!["sketchbook".to_owned(), "attic".to_owned()],
        };
        let text = digest_text(&digest);
        assert!(text.starts_with("You are the chat of Roadmap Wrangler"));
        assert!(text.contains("Today is 2026-11-11."));
        assert!(text.contains("house-style.md; the /roadmap-wrangler:status skill"));
        // The commands as the rules match them, and the actions' form (EP-24b).
        assert!(text.contains(
            "What you can run, with Bash, each exactly as written here and alone on its line \
             (no pipe, no redirect, no other command): `crw roadmap <project>`, `crw tasks \
             <project>`, `crw sessions <project>` with <project> the name of one readable \
             project below (in double quotes when it holds a space), and `crw sessions`, `crw \
             mail`, `crw notifications`, `crw usage`, `crw autopilot`."
        ));
        assert!(text.contains("```actions\nadopt <folder>\nformalise <project>\nmigrate <project>\ncompose <project>: <the message's words>\n```"));
        assert!(
            !text.contains("You may also run"),
            "no writing command is admitted"
        );
        assert!(text.contains(
            "Folders of the data root not yet adopted (an Adopt action registers one): \
             sketchbook, attic."
        ));
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
        assert!(text.contains(
            "- tundra (unformalised): C:\\Data\\tundra. No roadmap the parser reads. 3 open \
                 items (no owner-tasks.md: items in older forms, a risks list among them, as \
                 crw tasks lists them), 0 open inbox messages."
        ));
        assert!(text.contains("creating roadmap/inbox.md when the project has none"));
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
            readable: vec!["alpha".to_owned()],
            bin: Some(scratch.0.join("bin")),
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
            &Pick::default(),
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
        assert_eq!(
            after("--tools"),
            ["Read,Glob,Grep,Skill,AskUserQuestion,Bash"]
        );
        // The rules, one argument each after the flag, up to the next flag (EP-24b): exact
        // for the project commands on each readable name, bare for the rest, no rule for a
        // data repository's name nor a prefix one.
        let at = args.iter().position(|arg| arg == "--allowedTools").unwrap();
        let rules: Vec<&str> = args[at + 1..]
            .iter()
            .take_while(|arg| !arg.starts_with("--"))
            .map(String::as_str)
            .collect();
        assert_eq!(
            rules,
            [
                "Bash(crw roadmap alpha)",
                "Bash(crw tasks alpha)",
                "Bash(crw sessions alpha)",
                "Bash(crw sessions)",
                "Bash(crw mail)",
                "Bash(crw notifications)",
                "Bash(crw usage)",
                "Bash(crw autopilot)",
            ]
        );
        assert!(args[at + 1 + rules.len()].starts_with("--"));
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
        // The neighbours' CLAUDE.md files stay unloaded: the key leaves the environment, and
        // the child's PATH starts with the bin folder, the rest inherited (EP-24b).
        assert_eq!(line.env.len(), 2, "{:?}", line.env);
        assert_eq!(line.env[0], (OsString::from("ANTHROPIC_API_KEY"), None));
        assert_eq!(line.env[1].0, OsString::from("PATH"));
        let path = line.env[1]
            .1
            .clone()
            .unwrap()
            .to_string_lossy()
            .into_owned();
        let separator = if cfg!(windows) { ';' } else { ':' };
        assert!(
            path.starts_with(&format!("{}{separator}", scratch.0.join("bin").display())),
            "{path}"
        );
        assert!(path.len() > scratch.0.join("bin").display().to_string().len() + 1);

        // A later turn resumes the session with the owner's message, under the model and
        // effort the conversation's picker holds (EP-24a); no bin beside the window: no PATH.
        let places = Places {
            cwd: cwd.clone(),
            folders: vec![alpha.clone()],
            readable: vec!["alpha".to_owned()],
            bin: None,
            digest: Appended {
                file: digest_file(&cwd, "0e1d"),
                text: "A fresh digest.".to_owned(),
            },
            ledger: None,
        };
        let resumed = Turn::Resume {
            session_id: "0e1d".to_owned(),
        };
        let pick = Pick {
            model: "fable".to_owned(),
            effort: None,
        };
        assert_eq!(pick.words(), "fable at the CLI's default effort");
        assert_eq!(Pick::default().words(), "claude-opus-5-5 at medium");
        let turn = super::launch(
            "What does EP-24 plan?",
            "And EP-24a?",
            &resumed,
            places,
            &pick,
        );
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
        assert!(
            args.windows(2)
                .any(|pair| pair[0] == "--model" && pair[1] == "fable")
        );
        assert!(!args.iter().any(|arg| arg == "--effort"));
        assert_eq!(line.env, [(OsString::from("ANTHROPIC_API_KEY"), None)]);

        // A claude without --allowedTools runs the turn without the rules and says so: each
        // command then prompts, and the window refuses it.
        let without = Flags::parse(&HELP.replace("--allowedTools, --allowed-tools", "--other"));
        let line = crate::session::launch::command_line(&turn, &without, "0e1d", []).unwrap();
        assert!(!line.args.iter().any(|arg| arg == "--allowedTools"));
        assert_eq!(line.dropped.len(), 1, "{:?}", line.dropped);
        assert!(
            line.dropped[0].starts_with(
                "--allowedTools: this claude does not list it, so each of the 8 commands"
            ),
            "{}",
            line.dropped[0]
        );

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

    /// The rules of what a turn may run (EP-24b): exact per readable project, bare for the
    /// rest, and no writing command while the owner admits none.
    #[test]
    fn the_rules_name_each_readable_project_exactly() {
        let rules = allowed_rules(&["alpha".to_owned(), "beta two".to_owned()]);
        assert_eq!(
            rules,
            [
                "Bash(crw roadmap alpha)",
                "Bash(crw tasks alpha)",
                "Bash(crw sessions alpha)",
                "Bash(crw roadmap \"beta two\")",
                "Bash(crw tasks \"beta two\")",
                "Bash(crw sessions \"beta two\")",
                "Bash(crw sessions)",
                "Bash(crw mail)",
                "Bash(crw notifications)",
                "Bash(crw usage)",
                "Bash(crw autopilot)",
            ]
        );
        assert!(rules.iter().all(|rule| !rule.contains(":*")));
        assert_eq!(allowed_rules(&[]).len(), BARE_COMMANDS.len());
    }

    /// A reply's actions block (EP-24b): each form read, the block taken out of the text, the
    /// other fences kept, an unknown line kept as such, a block left open counted whole.
    #[test]
    fn the_proposals_are_read_out_of_the_reply() {
        let reply = "## Two projects\n\nquarry has no `roadmap/owner-tasks.md`; attic is a folder \
                     of the data root the registry does not list.\n\n```toml\nkeep = true\n```\n\n\
                     ```actions\nformalise quarry\nAdopt attic\nmigrate @orchard\n\
                     compose harbor: EP-4's band default: ask the owner before the layout\n\
                     compose lantern the season names\nstart harbor\n\n```\n";
        let (text, proposed) = proposals(reply);
        assert_eq!(
            text,
            "## Two projects\n\nquarry has no `roadmap/owner-tasks.md`; attic is a folder of \
             the data root the registry does not list.\n\n```toml\nkeep = true\n```"
        );
        assert_eq!(
            proposed,
            [
                Proposed::Formalise {
                    project: "quarry".to_owned()
                },
                Proposed::Adopt {
                    folder: "attic".to_owned()
                },
                Proposed::Migrate {
                    project: "orchard".to_owned()
                },
                Proposed::Compose {
                    project: "harbor".to_owned(),
                    words: "EP-4's band default: ask the owner before the layout".to_owned()
                },
                Proposed::Compose {
                    project: "lantern".to_owned(),
                    words: "the season names".to_owned()
                },
                Proposed::Unknown {
                    line: "start harbor".to_owned()
                },
            ]
        );
        assert_eq!(proposed[0].label(), "Formalise quarry…");
        assert_eq!(proposed[1].label(), "Adopt attic…");
        assert_eq!(proposed[2].label(), "Migrate orchard…");
        assert_eq!(proposed[3].label(), "Compose to harbor…");
        assert_eq!(proposed[5].label(), "start harbor");
        assert_eq!(proposed[3].target(), Some("harbor"));
        assert_eq!(proposed[5].target(), None);

        // No block: the text as it is, nothing proposed; a block in the middle, with tildes,
        // leaves the text around it; one left open at the end counts whole.
        let plain = "Nothing to do.\n\n```sh\ncrw tasks harbor\n```";
        assert_eq!(proposals(plain), (plain.to_owned(), Vec::new()));
        let (text, proposed) = proposals("Before.\n\n~~~Actions\nformalise quarry\n~~~\n\nAfter.");
        assert_eq!(text, "Before.\n\n\nAfter.");
        assert_eq!(proposed.len(), 1);
        let (text, proposed) = proposals("The step:\n\n```actions\nadopt attic");
        assert_eq!(text, "The step:");
        assert_eq!(
            proposed,
            [Proposed::Adopt {
                folder: "attic".to_owned()
            }]
        );
        // A verb with nothing after it is unknown, not an empty target.
        let (_, proposed) = proposals("```actions\nformalise\n```");
        assert_eq!(
            proposed,
            [Proposed::Unknown {
                line: "formalise".to_owned()
            }]
        );
        // A longer fence quoting the form closes only with a run as long (the review's
        // finding); the real block after it is read. A list marker before the verb goes.
        let (text, proposed) = proposals(
            "The form:\n\n````markdown\n```actions\nformalise <project>\n```\n````\n\n\
             ```actions\n- formalise quarry\n2. adopt attic\n```",
        );
        assert_eq!(
            text,
            "The form:\n\n````markdown\n```actions\nformalise <project>\n```\n````"
        );
        assert_eq!(
            proposed,
            [
                Proposed::Formalise {
                    project: "quarry".to_owned()
                },
                Proposed::Adopt {
                    folder: "attic".to_owned()
                },
            ]
        );
        // A long unknown line's label is cut, so that no chip widens the pane.
        let long = Proposed::Unknown {
            line: "start a session on quarry's next brief and tell me when it is done".to_owned(),
        };
        assert!(long.label().chars().count() <= LABEL_CHARS);
        assert!(
            long.label()
                .starts_with("start a session on quarry's next brief")
        );
        assert!(long.label().ends_with('…'));
        assert_eq!(spelled("alpha"), "alpha");
        assert_eq!(spelled("beta two"), "\"beta two\"");
    }

    /// The appended text (EP-24b): the digest alone, or with a mention's context after it.
    #[test]
    fn the_appended_text_carries_a_mentions_context() {
        assert_eq!(appended("The digest.", None), "The digest.");
        let text = appended("The digest.", Some("alpha: its next row."));
        assert!(text.starts_with("The digest.\n\nThe owner's message names a project with @."));
        assert!(text.ends_with(":\n\nalpha: its next row."));
    }
}
