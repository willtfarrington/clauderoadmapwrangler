//! The prompt hook's mentions (IN-2, D-20; EP-20b): an `@<project>` in a prompt names a
//! registered project, whose `CLAUDE.md` and next row `crw hook` prints on the prompt's
//! `UserPromptSubmit`, which Claude Code adds to the session's context. The project is found as
//! the mailman finds a receiver, by the registry's name in any case. A mention inside code (a
//! backtick span, a fenced block) or inside pasted content is no mention, nor is an e-mail
//! address or a file path, and the session's own project is left out: its `CLAUDE.md` is loaded
//! already. The texts share [`BUDGET`], under Claude Code's cap on a hook's output.

use std::fs;
use std::path::Path;

use crate::prompt;
use crate::roadmap::snapshot::Snapshot;
use crate::roadmap::{readme, table};
use crate::sessions::{Registered, project_of};

/// The most projects one prompt is given.
pub const LIMIT: usize = 3;

/// The characters the whole context may take: Claude Code caps a hook's output at 10,000 and
/// beyond it passes only a preview, which Claude is not asked to read.
pub const BUDGET: usize = 9_000;

/// The tenths of a project's share its `CLAUDE.md` gets when both texts are longer than theirs.
const RULES_TENTHS: usize = 6;

/// What a cut text ends with, its file named after it.
const CUT: &str = "\n[cut here to keep within the hook's limit: the rest is in ";

/// The names a prompt mentions with `@`, in its order and each once (case aside): outside code
/// and pasted content, after the start of a line, a space, an opening bracket or quote, or an
/// emphasis mark; a name of letters, digits, `-`, `_` and `.`, its trailing `.`, `-` and `_`
/// dropped. A name followed by `/` or `\` is a file path, which Claude Code reads itself.
pub fn names(prompt: &str) -> Vec<String> {
    let mut found: Vec<String> = Vec::new();
    let mut fence: Option<&str> = None;
    let mut pasted = false;
    for line in prompt.lines() {
        let trimmed = line.trim_start();
        if pasted {
            pasted = !trimmed.starts_with("</pasted_content");
            continue;
        }
        if fence.is_none() && trimmed.starts_with("<pasted_content") {
            pasted = true;
            continue;
        }
        if let Some(open) = fence {
            if trimmed.starts_with(open) {
                fence = None;
            }
            continue;
        }
        if let Some(open) = ["```", "~~~"].into_iter().find(|f| trimmed.starts_with(f)) {
            fence = Some(open);
            continue;
        }
        for name in in_line(&blank_code(line)) {
            if !found.iter().any(|seen| seen.eq_ignore_ascii_case(&name)) {
                found.push(name);
            }
        }
    }
    found
}

/// `line` with its backtick code spans blanked out; a backtick run with no closing run of the
/// same length is text.
fn blank_code(line: &str) -> String {
    let chars: Vec<char> = line.chars().collect();
    let mut out = String::with_capacity(line.len());
    let mut i = 0;
    while i < chars.len() {
        if chars[i] != '`' {
            out.push(chars[i]);
            i += 1;
            continue;
        }
        let run = chars[i..].iter().take_while(|c| **c == '`').count();
        let mut j = i + run;
        let mut close = None;
        while j < chars.len() {
            if chars[j] == '`' {
                let other = chars[j..].iter().take_while(|c| **c == '`').count();
                if other == run {
                    close = Some(j + other);
                    break;
                }
                j += other;
            } else {
                j += 1;
            }
        }
        match close {
            Some(end) => {
                out.extend(std::iter::repeat_n(' ', end - i));
                i = end;
            }
            None => {
                out.extend(std::iter::repeat_n('`', run));
                i += run;
            }
        }
    }
    out
}

/// The `@` names of one line whose code is blanked.
fn in_line(line: &str) -> Vec<String> {
    let chars: Vec<char> = line.chars().collect();
    let mut names = Vec::new();
    for (at, c) in chars.iter().enumerate() {
        if *c != '@' {
            continue;
        }
        let opens = at
            .checked_sub(1)
            .map(|before| chars[before])
            .is_none_or(|before| {
                before.is_whitespace() || "([{\"'\u{201c}\u{2018}*_".contains(before)
            });
        if !opens {
            continue;
        }
        let rest = &chars[at + 1..];
        let length = rest
            .iter()
            .take_while(|c| c.is_alphanumeric() || "-_.".contains(**c))
            .count();
        if matches!(rest.get(length), Some('/' | '\\')) {
            continue;
        }
        let name: String = rest[..length].iter().collect();
        let name = name.trim_end_matches(['.', '-', '_']);
        if !name.is_empty() {
            names.push(name.to_owned());
        }
    }
    names
}

/// The context a prompt's mentions give a session working in `cwd`: for each registered project
/// it names (at most [`LIMIT`], the session's own left out), a framing line, its `CLAUDE.md` and
/// its next row with the brief notes, cut to share [`BUDGET`]; `None` when it names none.
pub fn context(prompt: &str, cwd: &Path, projects: &[Registered]) -> Option<String> {
    let own = project_of(projects, cwd);
    let mut chosen: Vec<&Registered> = Vec::new();
    let mut more: Vec<&str> = Vec::new();
    for name in names(prompt) {
        let Some(project) = projects
            .iter()
            .find(|project| project.name.eq_ignore_ascii_case(&name))
        else {
            continue;
        };
        if own.as_deref() == Some(project.name.as_str())
            || chosen.iter().any(|seen| seen.name == project.name)
        {
            continue;
        }
        if chosen.len() < LIMIT {
            chosen.push(project);
        } else if !more.contains(&project.name.as_str()) {
            more.push(&project.name);
        }
    }
    if chosen.is_empty() {
        return None;
    }
    let tail = (!more.is_empty()).then(|| {
        format!(
            "The prompt names {} too, past the {LIMIT} projects a prompt is given: read their \
             CLAUDE.md files if they matter.",
            more.iter()
                .map(|name| format!("@{name}"))
                .collect::<Vec<_>>()
                .join(", ")
        )
    });
    let budget = BUDGET.saturating_sub(tail.as_ref().map_or(0, |tail| tail.chars().count() + 2));
    let share = (budget / chosen.len()).saturating_sub(2);
    let mut sections: Vec<String> = chosen
        .iter()
        .map(|project| section(project, share))
        .collect();
    sections.extend(tail);
    Some(sections.join("\n\n"))
}

/// One project's part of the context, at most `share` characters.
fn section(project: &Registered, share: usize) -> String {
    let folder = &project.folder;
    let name = &project.name;
    let head = format!(
        "The prompt mentions @{name}, a project Roadmap Wrangler knows at {} (its prompt hook, \
         crw EP-20b). Its rules and its next brief follow for reference: they are that \
         project's, not this session's, whose own CLAUDE.md still governs. Read that project's \
         files only; its plan changes through a message in its roadmap/inbox.md.",
        folder.display()
    );
    // Where the launch finds it (`prompt::claude_md`), its path kept to name it.
    let rules = [
        folder.join("CLAUDE.md"),
        folder.join(".claude").join("CLAUDE.md"),
    ]
    .into_iter()
    .find_map(|path| {
        let text = fs::read_to_string(&path).ok()?;
        Some((path, text.trim().to_owned()))
    });
    let readme = readme(folder);
    let row = fs::read_to_string(&readme)
        .map_err(|_| format!("It has no {} the tool reads.", readme.display()))
        .and_then(|text| {
            let snapshot = Snapshot {
                folder: folder.clone(),
                tree: Some(table::parse(&text)),
                ..Snapshot::default()
            };
            prompt::next_of(&snapshot)
                .map(|(roadmap, row)| prompt::row_parts(roadmap, row))
                .map_err(|why| format!("It has no next brief: {why}."))
        });
    let rules_label = match &rules {
        Some((path, _)) => format!("{name}'s CLAUDE.md ({}):", path.display()),
        None => format!("{name} has no CLAUDE.md."),
    };
    let (row_head, note) = match &row {
        Ok(parts) => (
            format!(
                "{name}'s next brief, the first unchecked row of roadmap/README.md in table \
                 order:\n{}\nIts brief notes, verbatim:",
                prompt::row_lines(parts)
            ),
            parts.note.as_str(),
        ),
        Err(why) => (why.clone(), ""),
    };
    let fixed = [&head, &rules_label, &row_head]
        .iter()
        .map(|text| text.chars().count() + 2)
        .sum::<usize>();
    let rules_text = rules.as_ref().map_or("", |(_, text)| text.as_str());
    let (rules_max, note_max) = shares(
        rules_text.chars().count(),
        note.chars().count(),
        share.saturating_sub(fixed + 2),
    );
    let mut out = vec![head, rules_label];
    if let Some((path, text)) = &rules {
        out.push(cut(text, rules_max, path));
    }
    out.push(row_head);
    if !note.is_empty() {
        out.push(cut(note, note_max, &readme));
    }
    out.join("\n\n")
}

/// How much of the rules and of the note fit in `room`: their shares, the room one leaves
/// going to the other.
fn shares(rules: usize, note: usize, room: usize) -> (usize, usize) {
    let rules_quota = room * RULES_TENTHS / 10;
    let note_quota = room - rules_quota;
    if rules <= rules_quota {
        (rules, note.min(room - rules))
    } else if note <= note_quota {
        (rules.min(room - note), note)
    } else {
        (rules_quota, note_quota)
    }
}

/// `text` in at most `max` characters: whole when it fits, else cut at a line's end where one
/// is near, saying where the rest is.
fn cut(text: &str, max: usize, file: &Path) -> String {
    if text.chars().count() <= max {
        return text.to_owned();
    }
    let marker = format!("{CUT}{}]", file.display());
    let keep = max.saturating_sub(marker.chars().count());
    let end = text
        .char_indices()
        .nth(keep)
        .map_or(text.len(), |(at, _)| at);
    let mut kept = &text[..end];
    if let Some(line_end) = kept.rfind('\n').filter(|at| *at >= end / 2) {
        kept = &kept[..line_end];
    }
    format!("{}{marker}", kept.trim_end())
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;
    use crate::files::tests::Scratch;

    const README: &str = "\
# beta — Roadmap

## Phase P1 — Build (planned 2026-10-01)

| # | Brief | Size | Depends on | Core | Done |
|---|---|---|---|---|---|
| EP-1 | Toolchain | M | — | core | ☑ `b2c3d4e` |
| EP-2 | Parser | S | EP-1 | core | ☐ |

**Brief notes.**

- **EP-2.** Read the tables.
";

    fn registered(root: &Path, names: &[&str]) -> Vec<Registered> {
        names
            .iter()
            .map(|name| Registered {
                name: (*name).to_owned(),
                folder: root.join(name),
                revert: true,
            })
            .collect()
    }

    fn project(root: &Path, name: &str, claude_md: Option<&str>, readme: Option<&str>) {
        let folder = root.join(name);
        fs::create_dir_all(folder.join("roadmap")).unwrap();
        if let Some(text) = claude_md {
            fs::write(folder.join("CLAUDE.md"), text).unwrap();
        }
        if let Some(text) = readme {
            fs::write(folder.join("roadmap").join("README.md"), text).unwrap();
        }
    }

    #[test]
    fn a_prompt_names_projects_outside_code_and_pasted_text() {
        assert_eq!(
            names("@beta what is next? And (@Gamma), @beta again, \"@delta\" **@epsilon**."),
            ["beta", "Gamma", "delta", "epsilon"]
        );
        // Trailing punctuation goes; an e-mail address, a bare @ and a path are no mentions.
        assert_eq!(
            names("ask @beta. then @gamma-, @zeta_:"),
            ["beta", "gamma", "zeta"]
        );
        assert!(names("mail owner@example.com or @ alone").is_empty());
        assert!(names("read @beta/roadmap/README.md and @gamma\\CLAUDE.md").is_empty());
        // Code spans, fenced blocks and pasted content hold no mentions.
        assert_eq!(
            names("see `@beta` and ``a ` @gamma`` but @delta\n```\n@epsilon\n```\n@zeta"),
            ["delta", "zeta"]
        );
        assert_eq!(
            names(
                "look:\n<pasted_content id=\"p1\">\n@beta says hi\n</pasted_content id=\"p1\">\n@gamma"
            ),
            ["gamma"]
        );
        // A backtick with no closing run is text.
        assert_eq!(names("a ` then @beta"), ["beta"]);
        // `@<project>`, as a brief writes it, names nothing.
        assert!(names("an @<project> in a prompt").is_empty());
    }

    #[test]
    fn the_context_gives_the_rules_and_the_next_row() {
        let scratch = Scratch::new("mentions-context");
        let root = &scratch.0;
        project(
            root,
            "alpha",
            Some("# alpha\n\n1. Own rules."),
            Some(README),
        );
        project(
            root,
            "beta",
            Some("# beta — rules\n\n1. **Read first.**\n"),
            Some(README),
        );
        project(root, "gamma", None, None);
        project(
            root,
            "delta",
            Some("# delta"),
            Some("# delta\n\nNo table.\n"),
        );
        let projects = registered(root, &["alpha", "beta", "gamma", "delta"]);
        let cwd = root.join("alpha").join("crates");

        let text = context("Compare with @BETA please", &cwd, &projects).unwrap();
        assert!(
            text.starts_with("The prompt mentions @beta, a project Roadmap Wrangler knows at ")
        );
        assert!(text.contains("not this session's, whose own CLAUDE.md still governs"));
        assert!(
            text.contains("# beta — rules\n\n1. **Read first.**"),
            "{text}"
        );
        assert!(
            text.contains(
                "- EP-2 · S · depends on EP-1 · core · phase P1 — Build\n- Parser\nIts brief notes, verbatim:\n\n**EP-2.** Read the tables."
            ),
            "{text}"
        );
        // The session's own project is loaded already; an unknown name is nothing.
        assert_eq!(context("@alpha and @omega", &cwd, &projects), None);
        // A project without a CLAUDE.md or a README, and one whose README has no table.
        let text = context("@gamma @delta", &cwd, &projects).unwrap();
        assert!(text.contains("gamma has no CLAUDE.md."), "{text}");
        assert!(text.contains("It has no "), "{text}");
        assert!(
            text.contains("It has no next brief: its roadmap has no phase table"),
            "{text}"
        );
    }

    #[test]
    fn long_texts_are_cut_to_stay_under_the_cap() {
        let scratch = Scratch::new("mentions-cap");
        let root = &scratch.0;
        let long_rules = "A rule that goes on.\n".repeat(2000);
        let long_note = format!(
            "{}\n",
            "- **EP-2.** A note that goes on and on.".repeat(500)
        );
        let readme = README.replace("- **EP-2.** Read the tables.\n", &long_note);
        let names_list = ["b1", "b2", "b3", "b4", "b5"];
        for name in names_list {
            project(root, name, Some(&long_rules), Some(&readme));
        }
        let projects = registered(root, &names_list);
        let one = context("@b1", Path::new(r"C:\Elsewhere"), &projects).unwrap();
        assert!(one.chars().count() <= BUDGET, "{}", one.chars().count());
        assert!(one.contains("[cut here to keep within the hook's limit: the rest is in "));
        assert!(one.contains(&root.join("b1").join("CLAUDE.md").display().to_string()));
        let all = context("@b1 @b2 @b3 @b4 @b5", Path::new(r"C:\Elsewhere"), &projects).unwrap();
        assert!(all.chars().count() <= BUDGET, "{}", all.chars().count());
        assert_eq!(all.matches("The prompt mentions @").count(), LIMIT);
        assert!(all.ends_with(
            "The prompt names @b4, @b5 too, past the 3 projects a prompt is given: read their \
             CLAUDE.md files if they matter."
        ));
        // A short text keeps its whole; the room it leaves goes to the other.
        assert_eq!(shares(100, 5000, 1000), (100, 900));
        assert_eq!(shares(5000, 100, 1000), (900, 100));
        assert_eq!(shares(5000, 5000, 1000), (600, 400));
        // A cut ends at a line's end, within its count, and names the file.
        let cut_text = cut(&"one\ntwo\nthree\n".repeat(20), 100, &PathBuf::from("x.md"));
        assert!(cut_text.chars().count() <= 100, "{cut_text}");
        assert!(cut_text.starts_with("one\ntwo\nthree\n"), "{cut_text}");
        assert!(
            cut_text.ends_with(
                "\none\ntwo\n[cut here to keep within the hook's limit: the rest is in x.md]"
            ),
            "{cut_text}"
        );
        assert_eq!(cut("short", 100, Path::new("x.md")), "short");
    }
}
