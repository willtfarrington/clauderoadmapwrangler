//! The Markdown the roadmap files write inside their lines, drawn rather than shown as source:
//! `**bold**` strong, `` `code` `` in the code style, `~~struck~~` struck through; and a bullet's
//! continuation lines joined into one paragraph, so the window wraps them at its own width.

use eframe::egui::{self, Align, FontSelection, RichText, text::LayoutJob};

/// The inline marks in force over a run of text.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Marks {
    pub bold: bool,
    pub code: bool,
    pub struck: bool,
}

/// `text` split at its markup into runs, each with the marks in force; a mark with no closing
/// partner stays as written, and nothing is marked inside code.
pub fn spans(text: &str) -> Vec<(String, Marks)> {
    let mut runs: Vec<(String, Marks)> = Vec::new();
    let mut marks = Marks::default();
    let mut current = String::new();
    let mut rest = text;
    while let Some(c) = rest.chars().next() {
        let toggle = if marks.code {
            rest.starts_with('`').then_some("`")
        } else {
            ["**", "~~", "`"].into_iter().find(|mark| {
                rest.starts_with(mark)
                    && (is_open(marks, mark) || rest[mark.len()..].contains(mark))
            })
        };
        match toggle {
            Some(mark) => {
                if !current.is_empty() {
                    runs.push((std::mem::take(&mut current), marks));
                }
                match mark {
                    "**" => marks.bold = !marks.bold,
                    "~~" => marks.struck = !marks.struck,
                    _ => marks.code = !marks.code,
                }
                rest = &rest[mark.len()..];
            }
            None => {
                current.push(c);
                rest = &rest[c.len_utf8()..];
            }
        }
    }
    if !current.is_empty() {
        runs.push((current, marks));
    }
    runs
}

/// Whether `mark` is the one open now, so it closes here.
fn is_open(marks: Marks, mark: &str) -> bool {
    match mark {
        "**" => marks.bold,
        "~~" => marks.struck,
        _ => marks.code,
    }
}

/// A list item's text with its continuation lines joined by spaces; a line that opens another
/// list item, or follows a blank line, starts a new line.
pub fn reflow(text: &str) -> String {
    let mut out = String::new();
    let mut blank = false;
    for line in text.lines().map(str::trim) {
        if line.is_empty() {
            blank = true;
            continue;
        }
        let item = line.starts_with("- ")
            || line.starts_with("* ")
            || line
                .split_once(". ")
                .is_some_and(|(n, _)| !n.is_empty() && n.bytes().all(|b| b.is_ascii_digit()));
        if !out.is_empty() {
            out.push_str(match (blank, item) {
                (true, _) => "\n\n",
                (false, true) => "\n",
                (false, false) => " ",
            });
        }
        out.push_str(line);
        blank = false;
    }
    out
}

/// How a run of markup is drawn, before its own marks.
#[derive(Clone, Copy, Debug, Default)]
pub struct Look {
    pub small: bool,
    pub weak: bool,
}

/// `text`'s markup drawn in the label's style.
pub fn job(ui: &egui::Ui, text: &str, look: Look) -> LayoutJob {
    let mut job = LayoutJob::default();
    for (run, marks) in spans(text) {
        let mut rich = RichText::new(run);
        if look.small {
            rich = rich.small();
        }
        if look.weak && !marks.bold {
            rich = rich.weak();
        }
        if marks.code {
            rich = rich.code();
        }
        if marks.bold {
            rich = rich.strong();
        }
        if marks.struck {
            rich = rich.strikethrough();
        }
        rich.append_to(&mut job, ui.style(), FontSelection::Default, Align::Center);
    }
    job
}

/// A wrapped label of `text` with its markup drawn.
pub fn label(ui: &mut egui::Ui, text: &str, look: Look) -> egui::Response {
    let job = job(ui, text, look);
    ui.add(egui::Label::new(job).wrap())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The runs as text and a tag: b bold, c code, s struck.
    fn marked(text: &str) -> Vec<(String, &'static str)> {
        spans(text)
            .into_iter()
            .map(|(run, marks)| {
                let tag = match (marks.bold, marks.code, marks.struck) {
                    (false, false, false) => "",
                    (true, false, false) => "b",
                    (false, true, false) => "c",
                    (false, false, true) => "s",
                    (true, false, true) => "bs",
                    _ => "?",
                };
                (run, tag)
            })
            .collect()
    }

    fn runs(expected: &[(&str, &'static str)]) -> Vec<(String, &'static str)> {
        expected
            .iter()
            .map(|&(run, tag)| (run.to_owned(), tag))
            .collect()
    }

    #[test]
    fn splits_text_at_its_inline_marks() {
        assert_eq!(
            marked("**EP-4.** The `berths.toml` file"),
            runs(&[
                ("EP-4.", "b"),
                (" The ", ""),
                ("berths.toml", "c"),
                (" file", "")
            ])
        );
        assert_eq!(
            marked("~~**R-2** Gone.~~ **Resolved**"),
            runs(&[("R-2", "bs"), (" Gone.", "s"), (" ", ""), ("Resolved", "b")])
        );
        // Nothing is marked inside code; a lone mark stays as written.
        assert_eq!(
            marked("`a **b**` and 2 ** 3"),
            runs(&[("a **b**", "c"), (" and 2 ** 3", "")])
        );
        assert_eq!(marked("x ~~ y"), runs(&[("x ~~ y", "")]));
        assert_eq!(
            marked("ünïcode `ök`"),
            runs(&[("ünïcode ", ""), ("ök", "c")])
        );
        assert!(spans("").is_empty());
    }

    #[test]
    fn joins_continuation_lines_into_one_paragraph() {
        assert_eq!(
            reflow("The ledger view: one row\nper berth, a detail pane;\n  keys ↑ ↓."),
            "The ledger view: one row per berth, a detail pane; keys ↑ ↓."
        );
        assert_eq!(
            reflow("Two parts:\n- one\n- two\n\nAfter a blank line.\n1. first"),
            "Two parts:\n- one\n- two\n\nAfter a blank line.\n1. first"
        );
        assert_eq!(reflow("v2. notes stay"), "v2. notes stay");
    }
}
