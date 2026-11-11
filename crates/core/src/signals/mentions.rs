//! The prompt hook's mentions (IN-2, D-20; EP-20b): an `@<project>` in a prompt names a
//! registered project, whose next row and rules `crw hook` prints on the prompt's
//! `UserPromptSubmit`, which Claude Code adds to the session's context. The project is found as
//! the mailman finds a receiver, by the registry's name in any case. A mention inside code (a
//! backtick span, a fenced block) or inside pasted content is no mention, nor is an e-mail
//! address or a file path, and the session's own project is left out: its `CLAUDE.md` is loaded
//! already. The texts share [`BUDGET`], under Claude Code's cap on a hook's output. Since EP-39
//! (the D-20 addendum of 2026-11-11) a project's part holds, in this order: its next row and the
//! start of its brief note; its rules in short, each numbered rule's first sentence (a `CLAUDE.md`
//! without numbered rules: its first paragraphs), never one cut in the middle; then the paths of
//! its `CLAUDE.md` and its roadmap, for the rest.

use std::fs;
use std::path::Path;

use crate::prompt;
use crate::roadmap::markdown::{self, Block};
use crate::roadmap::snapshot::Snapshot;
use crate::roadmap::{readme, table};
use crate::sessions::{Registered, project_of};

/// The most projects one prompt is given.
pub const LIMIT: usize = 3;

/// The characters the whole context may take: Claude Code caps a hook's output at 10,000 and
/// beyond it passes only a preview, which Claude is not asked to read.
pub const BUDGET: usize = 9_000;

/// The tenths of a project's share its rules get when both texts are longer than theirs.
const RULES_TENTHS: usize = 6;

/// What a cut note ends with, its file named after it.
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
/// it names (at most [`LIMIT`], the session's own left out), a framing line, its next row with
/// the start of its brief notes, its rules in short and the paths of both files, cut to share
/// [`BUDGET`]; `None` when it names none.
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

/// One project's part of the context, at most `share` characters: the next row first, then
/// the rules in short, then the paths (D-20 addendum, 2026-11-11; EP-39).
fn section(project: &Registered, share: usize) -> String {
    let folder = &project.folder;
    let name = &project.name;
    let head = format!(
        "The prompt mentions @{name}, a project Roadmap Wrangler knows at {} (its prompt hook, \
         crw EP-20b). Its next brief and its rules in short follow for reference: they are that \
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
        Some((path, text))
    });
    let readme = readme(folder);
    let has_readme = readme.is_file();
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
    let (row_head, note) = match &row {
        Ok(parts) => (
            format!(
                "{name}'s next brief, the first unchecked row of roadmap/README.md in table \
                 order:\n{}",
                prompt::row_lines(parts)
            ),
            parts.note.as_str(),
        ),
        Err(why) => (why.clone(), ""),
    };
    // The note's label, by whether it is cut: the longer one is budgeted.
    const NOTE_WHOLE: &str = "Its brief notes, verbatim:";
    const NOTE_START: &str = "The start of its brief notes:";
    let (rules_label, rules_lines, unit) = match &rules {
        Some((path, text)) => {
            let (lines, how, unit) = in_short(text);
            (
                format!("{name}'s rules in short, {how} ({}):", path.display()),
                lines,
                unit,
            )
        }
        None => (format!("{name} has no CLAUDE.md."), Vec::new(), ""),
    };
    let mut rest: Vec<String> = Vec::new();
    if let Some((path, _)) = &rules {
        rest.push(path.display().to_string());
    }
    if has_readme {
        rest.push(readme.display().to_string());
    }
    let paths = (!rest.is_empty()).then(|| format!("The rest is in {}.", rest.join(" and ")));
    let mut fixed = [&head, &row_head, &rules_label]
        .into_iter()
        .chain(paths.as_ref())
        .map(|text| text.chars().count() + 2)
        .sum::<usize>();
    if !note.is_empty() {
        fixed += NOTE_START.chars().count() + 2;
    }
    let rules_count = rules_lines
        .iter()
        .map(|line| line.chars().count() + 1)
        .sum::<usize>()
        .saturating_sub(1);
    let (rules_max, note_max) = shares(
        rules_count,
        note.chars().count(),
        share.saturating_sub(fixed + 2),
    );
    let mut out = vec![head, row_head];
    if !note.is_empty() {
        let (text, whole) = cut_note(note, note_max, &readme);
        out.push(if whole { NOTE_WHOLE } else { NOTE_START }.to_owned());
        out.push(text);
    }
    out.push(rules_label);
    if let Some((path, _)) = &rules
        && !rules_lines.is_empty()
    {
        out.push(cut_units(&rules_lines, rules_max, unit, path));
    }
    out.extend(paths);
    out.join("\n\n")
}

/// A `CLAUDE.md` in short: with numbered rules at its top level, each rule's first sentence
/// under its number; without, its first paragraphs and bullets whole, each a line. The words
/// for the label, and the unit's name for the cut's count.
fn in_short(claude_md: &str) -> (Vec<String>, &'static str, &'static str) {
    let lines = markdown::lines(claude_md);
    let blocks = markdown::blocks(&lines);
    let item_text = |block_lines: &[markdown::Line<'_>], indent: usize| -> String {
        let text: Vec<&str> = block_lines
            .iter()
            .enumerate()
            .map(|(n, line)| {
                if n == 0 {
                    line.text.get(indent..).unwrap_or(line.text).trim_end()
                } else {
                    markdown::dedent(line.text, indent).trim_end()
                }
            })
            .collect();
        prompt::joined(text.join("\n").trim())
    };
    let numbered: Vec<String> = blocks
        .iter()
        .filter_map(|block| {
            let Block::Item { lines, indent } = block else {
                return None;
            };
            let first = lines.first()?;
            let marker = first.text.get(..*indent).unwrap_or("").trim();
            if !marker.starts_with(|c: char| c.is_ascii_digit()) {
                return None;
            }
            let text = item_text(lines, *indent);
            Some(format!("{marker} {}", first_sentence(&text)))
        })
        .collect();
    if !numbered.is_empty() {
        return (numbered, "each numbered rule's first sentence", "rules");
    }
    let paragraphs: Vec<String> = blocks
        .iter()
        .filter_map(|block| match block {
            Block::Item { lines, indent } => {
                let first = lines.first()?;
                let marker = first.text.get(..*indent).unwrap_or("").trim();
                Some(format!("{marker} {}", item_text(lines, *indent)))
            }
            Block::Paragraph { lines } => Some(prompt::joined(
                &lines
                    .iter()
                    .map(|line| line.text.trim())
                    .collect::<Vec<_>>()
                    .join("\n"),
            )),
            _ => None,
        })
        .filter(|text| !text.trim().is_empty())
        .collect();
    (paragraphs, "its first paragraphs", "paragraphs")
}

/// The first sentence of `text`: up to a `.`, `!` or `?` outside code and parentheses, with the
/// closing marks after it, that a space or the end follows; the whole when it has none.
fn first_sentence(text: &str) -> &str {
    let chars: Vec<(usize, char)> = text.char_indices().collect();
    let mut i = 0;
    let mut depth = 0usize;
    while i < chars.len() {
        let (_, c) = chars[i];
        match c {
            '`' => {
                // A code span: skipped to its closing run of the same length.
                let run = chars[i..].iter().take_while(|(_, c)| *c == '`').count();
                let mut j = i + run;
                let mut closed = None;
                while j < chars.len() {
                    if chars[j].1 == '`' {
                        let other = chars[j..].iter().take_while(|(_, c)| *c == '`').count();
                        if other == run {
                            closed = Some(j + other);
                            break;
                        }
                        j += other;
                    } else {
                        j += 1;
                    }
                }
                i = closed.unwrap_or(i + run);
                continue;
            }
            '(' | '[' => depth += 1,
            ')' | ']' => depth = depth.saturating_sub(1),
            '.' | '!' | '?' if depth == 0 => {
                let mut j = i + 1;
                while j < chars.len() && "*_\"'\u{201d}\u{2019})]".contains(chars[j].1) {
                    j += 1;
                }
                if j >= chars.len() || chars[j].1.is_whitespace() {
                    let end = chars.get(j).map_or(text.len(), |(at, _)| *at);
                    return &text[..end];
                }
            }
            _ => {}
        }
        i += 1;
    }
    text
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

/// `units` (a rule or a paragraph each) in at most `max` characters, one to a line: whole
/// units only, the ones left out counted and their file named.
fn cut_units(units: &[String], max: usize, unit: &str, file: &Path) -> String {
    let whole = units.join("\n");
    if whole.chars().count() <= max {
        return whole;
    }
    let marker = |left: usize| format!("[and {left} more {unit}, in {}]", file.display());
    let mut kept: Vec<&str> = Vec::new();
    let mut count = 0;
    for (n, line) in units.iter().enumerate() {
        let left = units.len() - n;
        let with = count + line.chars().count() + 1 + marker(left - 1).chars().count() + 1;
        if with > max {
            break;
        }
        kept.push(line);
        count += line.chars().count() + 1;
    }
    let left = units.len() - kept.len();
    if kept.is_empty() {
        return format!(
            "[the {unit} do not fit the hook's limit: read {}]",
            file.display()
        );
    }
    kept.push("");
    format!("{}{}", kept.join("\n"), marker(left))
}

/// `text` in at most `max` characters, and whether it is whole: cut at a sentence's end where
/// one lies in the second half, else at a line's end there, else at a space, saying where the
/// rest is.
fn cut_note(text: &str, max: usize, file: &Path) -> (String, bool) {
    if text.chars().count() <= max {
        return (text.to_owned(), true);
    }
    let marker = format!("{CUT}{}]", file.display());
    let keep = max.saturating_sub(marker.chars().count());
    let end = text
        .char_indices()
        .nth(keep)
        .map_or(text.len(), |(at, _)| at);
    let kept = &text[..end];
    let half = end / 2;
    let sentence_end = kept
        .char_indices()
        .filter(|(at, c)| {
            *at >= half
                && matches!(c, '.' | '!' | '?')
                && kept[at + 1..].starts_with(|next: char| next.is_whitespace())
        })
        .map(|(at, _)| at + 1)
        .next_back();
    let cut_at = sentence_end
        .or_else(|| kept.rfind('\n').filter(|at| *at >= half))
        .or_else(|| kept.rfind(' ').filter(|at| *at >= half))
        .unwrap_or(end);
    (format!("{}{marker}", kept[..cut_at].trim_end()), false)
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
        // The row first, its notes whole, then the rules in short, then the paths (EP-39).
        let row = "beta's next brief, the first unchecked row of roadmap/README.md in table \
                   order:\n- EP-2 · S · depends on EP-1 · core · phase P1 — Build\n- Parser\n\n\
                   Its brief notes, verbatim:\n\n**EP-2.** Read the tables.";
        assert!(text.contains(row), "{text}");
        let claude_md = root.join("beta").join("CLAUDE.md");
        let rules = format!(
            "beta's rules in short, each numbered rule's first sentence ({}):\n\n1. **Read first.**",
            claude_md.display()
        );
        assert!(text.contains(&rules), "{text}");
        assert!(text.find(row) < text.find(&rules));
        assert!(
            !text.contains("# beta — rules"),
            "the heading is not a rule: {text}"
        );
        assert!(
            text.ends_with(&format!(
                "The rest is in {} and {}.",
                claude_md.display(),
                root.join("beta")
                    .join("roadmap")
                    .join("README.md")
                    .display()
            )),
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
        // delta's CLAUDE.md is a heading alone: no rule, no paragraph, its path still named.
        assert!(
            text.contains("delta's rules in short, its first paragraphs ("),
            "{text}"
        );
        assert!(
            text.ends_with(&format!(
                "The rest is in {} and {}.",
                root.join("delta").join("CLAUDE.md").display(),
                root.join("delta")
                    .join("roadmap")
                    .join("README.md")
                    .display()
            )),
            "{text}"
        );
    }

    #[test]
    fn long_texts_are_cut_to_stay_under_the_cap() {
        let scratch = Scratch::new("mentions-cap");
        let root = &scratch.0;
        // Numbered rules, each of two sentences: the short form keeps the first of each.
        let long_rules: String = (1..=400)
            .map(|n| {
                format!("{n}. **Rule {n}:** it goes on\n   over a line. And then some more.\n")
            })
            .collect();
        let long_note = format!(
            "- **EP-2.** {}\n",
            "A note that goes on and on. ".repeat(500).trim_end()
        );
        let readme = README.replace("- **EP-2.** Read the tables.\n", &long_note);
        let names_list = ["b1", "b2", "b3", "b4", "b5"];
        for name in names_list {
            project(root, name, Some(&long_rules), Some(&readme));
        }
        let projects = registered(root, &names_list);
        let one = context("@b1", Path::new(r"C:\Elsewhere"), &projects).unwrap();
        assert!(one.chars().count() <= BUDGET, "{}", one.chars().count());
        // The note is cut at a sentence's end, and says so; the rules end on a whole rule,
        // joined, with the count of the rest and the file.
        assert!(one.contains("The start of its brief notes:\n\n**EP-2.** A note that goes on"));
        assert!(
            one.contains("and on.\n[cut here to keep within the hook's limit: the rest is in "),
            "{one}"
        );
        let claude_md = root.join("b1").join("CLAUDE.md");
        assert!(one.contains("1. **Rule 1:** it goes on over a line.\n2. **Rule 2:**"));
        assert!(!one.contains("And then some more"), "{one}");
        let marker_at = one.find("[and ").expect("the rules' count");
        let marker = &one[marker_at..];
        assert!(
            marker.starts_with("[and ")
                && marker.contains(&format!("more rules, in {}]", claude_md.display())),
            "{marker}"
        );
        // Every rule line before the marker is whole.
        let rules_at = one.find("rules in short, each").unwrap();
        for line in one[rules_at..marker_at].lines().skip(2) {
            assert!(line.is_empty() || line.ends_with("over a line."), "{line}");
        }
        assert!(one.ends_with(&format!(
            "The rest is in {} and {}.",
            claude_md.display(),
            root.join("b1").join("roadmap").join("README.md").display()
        )));
        let all = context("@b1 @b2 @b3 @b4 @b5", Path::new(r"C:\Elsewhere"), &projects).unwrap();
        assert!(all.chars().count() <= BUDGET, "{}", all.chars().count());
        assert_eq!(all.matches("The prompt mentions @").count(), LIMIT);
        assert_eq!(all.matches("The rest is in ").count(), LIMIT);
        assert!(all.ends_with(
            "The prompt names @b4, @b5 too, past the 3 projects a prompt is given: read their \
             CLAUDE.md files if they matter."
        ));
        // A short text keeps its whole; the room it leaves goes to the other.
        assert_eq!(shares(100, 5000, 1000), (100, 900));
        assert_eq!(shares(5000, 100, 1000), (900, 100));
        assert_eq!(shares(5000, 5000, 1000), (600, 400));
    }

    #[test]
    fn a_claude_md_without_numbered_rules_gives_its_paragraphs_whole() {
        let scratch = Scratch::new("mentions-paragraphs");
        let root = &scratch.0;
        let paragraphs: String = (1..=200)
            .map(|n| {
                format!(
                    "Paragraph {n} says\nsomething over two lines.\n\n- bullet {n}\n  goes on.\n\n"
                )
            })
            .collect();
        project(
            root,
            "beta",
            Some(&format!("# beta\n\n{paragraphs}")),
            Some(README),
        );
        let projects = registered(root, &["beta"]);
        let text = context("@beta", Path::new(r"C:\Elsewhere"), &projects).unwrap();
        assert!(text.chars().count() <= BUDGET);
        assert!(
            text.contains("beta's rules in short, its first paragraphs ("),
            "{text}"
        );
        assert!(
            text.contains("Paragraph 1 says something over two lines.\n- bullet 1 goes on.\n"),
            "{text}"
        );
        let marker_at = text.find("[and ").expect("the paragraphs' count");
        assert!(
            text[marker_at..].contains(" more paragraphs, in "),
            "{text}"
        );
        for line in text[text.find("rules in short, its").unwrap()..marker_at]
            .lines()
            .skip(2)
        {
            assert!(
                line.is_empty() || line.ends_with("two lines.") || line.ends_with("goes on."),
                "{line}"
            );
        }
    }

    #[test]
    fn the_short_form_and_the_cuts_keep_units_whole() {
        // The first sentence: code spans and parentheses skipped, closing marks kept.
        assert_eq!(
            first_sentence("**Read first:** `a.md` (D-1 … D-3; see `x.y`). Then more. And."),
            "**Read first:** `a.md` (D-1 … D-3; see `x.y`)."
        );
        assert_eq!(
            first_sentence("**Write the brief first.** Each session starts so."),
            "**Write the brief first.**"
        );
        assert_eq!(first_sentence("No end here"), "No end here");
        assert_eq!(
            first_sentence("Version 2.1.316 is the pin. Next."),
            "Version 2.1.316 is the pin."
        );
        assert_eq!(first_sentence("Is it? Yes."), "Is it?");
        // In short: numbered rules' first sentences under their numbers, joined over lines.
        let (lines, how, unit) = in_short(
            "# rules\n\n1. **One.** More of\n   one.\n2. Two goes\n   on. Then.\n\n- a bullet\n\nA paragraph.\n",
        );
        assert_eq!(lines, ["1. **One.**", "2. Two goes on."]);
        assert_eq!(
            (how, unit),
            ("each numbered rule's first sentence", "rules")
        );
        // Without numbered rules: paragraphs and bullets whole, headings and tables left out.
        let (lines, how, unit) = in_short(
            "# rules\n\nFirst para\nover lines.\n\n| a | b |\n|---|---|\n| 1 | 2 |\n\n- bullet\n  text.\n\nSecond.\n",
        );
        assert_eq!(
            lines,
            ["First para over lines.", "- bullet text.", "Second."]
        );
        assert_eq!((how, unit), ("its first paragraphs", "paragraphs"));
        assert!(in_short("# heading only\n").0.is_empty());
        // Units: whole lines only, the rest counted; none fitting says so.
        let units: Vec<String> = (1..=10).map(|n| format!("{n}. rule number {n}.")).collect();
        let file = PathBuf::from("x.md");
        assert_eq!(cut_units(&units, 1000, "rules", &file), units.join("\n"));
        let cut_text = cut_units(&units, 90, "rules", &file);
        assert!(cut_text.chars().count() <= 90, "{cut_text}");
        assert_eq!(
            cut_text,
            "1. rule number 1.\n2. rule number 2.\n3. rule number 3.\n[and 7 more rules, in x.md]"
        );
        assert_eq!(
            cut_units(&units, 10, "rules", &file),
            "[the rules do not fit the hook's limit: read x.md]"
        );
        // The note: whole when it fits, else at a sentence's end in the second half, else a
        // line's end, else a space.
        assert_eq!(cut_note("short", 100, &file), ("short".to_owned(), true));
        let note = "One sentence here. Two sentences here. Three sentences here. Four here. Five \
                    sentences here. Six sentences here.";
        assert!(note.chars().count() > 110);
        let (text, whole) = cut_note(note, 110, &file);
        assert!(!whole);
        assert!(text.chars().count() <= 110, "{text}");
        assert_eq!(
            text,
            "One sentence here. Two sentences here.\n[cut here to keep within the hook's limit: \
             the rest is in x.md]"
        );
        let (text, _) = cut_note(&"one\ntwo\nthree\n".repeat(20), 100, &file);
        assert!(text.chars().count() <= 100, "{text}");
        assert!(
            text.ends_with(
                "\none\ntwo\n[cut here to keep within the hook's limit: the rest is in x.md]"
            ),
            "{text}"
        );
        let (text, _) = cut_note(&"word ".repeat(100), 100, &file);
        assert!(
            text.chars().count() <= 100 && text.contains("word word\n[cut here"),
            "{text}"
        );
    }
}
