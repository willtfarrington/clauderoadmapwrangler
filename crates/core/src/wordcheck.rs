//! The word check (D-3, D-24; EP-42): a project's history read against the owner's list of words
//! that must not reach a public repository, as this repository's rule 3 reads one staged diff with
//! `grep -iFf`. Make public's dialog runs it before its confirmation ([`crate::actions`]), and
//! `crw act <project> make-public` previews it.
//!
//! One git child prints the history ([`command`]), in its turn through the actions' runner, and
//! this module reads its output in the process: the words are never passed to a child, and the
//! words, the lines that hold them and the list's contents are never shown beyond a count, never
//! logged and never written to the ledger. A [`Scan`] names the commits that hit by short hash
//! and subject, and a subject that itself holds a word is never shown.

use std::borrow::Cow;
use std::fmt;
use std::path::{Path, PathBuf};
use std::time::Duration;

use crate::actions::{Command, Exit, Runner};

/// How long the pass may take: this repository's 141 commits, 7 MB of diff, took two seconds in
/// November 2026.
pub const TIMEOUT: Duration = Duration::from_secs(120);

/// The separators of git's format: a commit's record starts with NUL, its short hash, subject and
/// raw message end with US, US and RS, and its diff follows.
const RECORD: char = '\0';
const FIELD: char = '\u{1f}';
const MESSAGE_END: char = '\u{1e}';

/// The owner's words, each a fixed string, lowercased. Its `Debug` prints the count alone.
#[derive(Clone, PartialEq, Eq)]
pub struct List {
    words: Vec<String>,
}

impl fmt::Debug for List {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "List({} words)", self.words.len())
    }
}

impl List {
    /// The words of a list's text, one fixed string a line as `grep -F -f` takes them: a line
    /// ending's `\r` and a leading byte-order mark dropped, and blank lines skipped, since an
    /// empty pattern would match every line.
    pub fn parse(text: &str) -> Self {
        let text = text.strip_prefix('\u{feff}').unwrap_or(text);
        let words = text
            .lines()
            .map(|line| line.strip_suffix('\r').unwrap_or(line))
            .filter(|line| !line.trim().is_empty())
            .map(str::to_lowercase)
            .collect();
        Self { words }
    }

    /// The list in the file at `path` ([`decode`]: UTF-8, or UTF-16 by its mark).
    pub fn read(path: &Path) -> Result<Self, String> {
        let bytes =
            std::fs::read(path).map_err(|e| format!("{} did not read: {e}", path.display()))?;
        decode(&bytes)
            .map(|text| Self::parse(&text))
            .map_err(|why| format!("{} {why}", path.display()))
    }

    /// The list a setting names: `None` while it names none.
    pub fn named(path: Option<PathBuf>) -> Option<Result<Self, String>> {
        path.map(|path| Self::read(&path))
    }

    /// How many words it holds.
    pub fn len(&self) -> usize {
        self.words.len()
    }

    pub fn is_empty(&self) -> bool {
        self.words.is_empty()
    }

    /// Whether `line` holds one of the words, case ignored.
    pub fn hits(&self, line: &str) -> bool {
        if self.words.is_empty() {
            return false;
        }
        let line = line.to_lowercase();
        self.words.iter().any(|word| line.contains(word.as_str()))
    }
}

/// A list file's text: UTF-16 by its byte-order mark, as Windows PowerShell 5.1's `>` and
/// Notepad's "Unicode" write it; else UTF-8, an odd byte replaced. A file with NUL bytes and no
/// such mark is refused (EP-42's review), since none of its words could ever match.
fn decode(bytes: &[u8]) -> Result<String, String> {
    let utf16 = |rest: &[u8], unit: fn([u8; 2]) -> u16| {
        let units: Vec<u16> = rest
            .as_chunks::<2>()
            .0
            .iter()
            .map(|pair| unit(*pair))
            .collect();
        String::from_utf16_lossy(&units)
    };
    match bytes {
        [0xFF, 0xFE, rest @ ..] => Ok(utf16(rest, u16::from_le_bytes)),
        [0xFE, 0xFF, rest @ ..] => Ok(utf16(rest, u16::from_be_bytes)),
        _ if bytes.contains(&0) => Err(
            "holds NUL bytes and no UTF-16 mark, so no word in it could match: save it as UTF-8"
                .to_owned(),
        ),
        _ => Ok(String::from_utf8_lossy(bytes).into_owned()),
    }
}

/// A commit whose message or added lines hold a listed word.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Hit {
    /// Its short hash.
    pub short: String,
    /// Its subject, or `None` when the subject itself holds a word, which is then never shown.
    pub subject: Option<String>,
    /// How many of its lines hold a word: its message's and its diff's added ones.
    pub lines: usize,
}

/// What stands for a subject that holds a listed word.
pub const MASKED: &str = "(its subject holds a listed word)";

impl Hit {
    /// `a1b2c3d Add the parser (2 lines)`, the subject masked when it holds a word.
    pub fn words(&self) -> String {
        let lines = if self.lines == 1 {
            "1 line".to_owned()
        } else {
            format!("{} lines", self.lines)
        };
        format!(
            "{} {} ({lines})",
            self.short,
            self.subject.as_deref().unwrap_or(MASKED)
        )
    }
}

/// What one pass over a history found.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Scan {
    /// The commits read.
    pub commits: usize,
    /// The list's words, counted.
    pub words: usize,
    /// The commits that hit, the newest first.
    pub hits: Vec<Hit>,
}

impl Scan {
    /// `141 commits read against a list of 37 words: none hit`, or how many did.
    pub fn summary(&self) -> String {
        let commits = plural(self.commits, "commit");
        let words = plural(self.words, "word");
        match self.hits.len() {
            0 => format!("{commits} read against a list of {words}: none hit"),
            n => format!("{commits} read against a list of {words}: {n} hit"),
        }
    }
}

/// `1 word`, `27 words`.
pub fn plural(count: usize, word: &str) -> String {
    if count == 1 {
        format!("1 {word}")
    } else {
        format!("{count} {word}s")
    }
}

/// The git child of a pass over `folder`'s history: every commit reachable from the branches, the
/// tags and the remote-tracking branches (what GitHub holds as last fetched, and what a later
/// push would add), never the stash; each with its raw message and its diff against its parent
/// with no context, the root commit's whole tree too (whatever `log.showRoot` says), renames read
/// as a removal and an addition so that every line a commit's tree gains counts, a merge's as its
/// dense combined diff; paths as they are, no colour, no external diff, no text conversion, no
/// signature and no note.
pub fn command(folder: &Path) -> Command {
    let folder = folder.display().to_string();
    let args = [
        "-C",
        &folder,
        "-c",
        "core.quotepath=false",
        "log",
        "--branches",
        "--tags",
        "--remotes",
        "--root",
        "--no-color",
        "--no-ext-diff",
        "--no-textconv",
        "--no-renames",
        "--no-show-signature",
        "--no-notes",
        "-p",
        "-U0",
        "--cc",
        "--format=%x00%h%x1f%s%x1f%B%x1e",
    ];
    Command {
        program: PathBuf::from("git"),
        args: args.iter().map(|arg| (*arg).to_owned()).collect(),
        long: false,
        timeout: TIMEOUT,
        folder: None,
    }
}

/// One pass over `folder`'s history against `list` through `runner`: git in its turn (D-29), its
/// output read here. A failure is said in [`failure`]'s words, which never hold the history.
pub fn history(folder: &Path, list: &List, runner: &dyn Runner) -> Result<Scan, String> {
    let exit = runner.run(&command(folder))?;
    if exit.code != Some(0) {
        return Err(failure(&exit, list));
    }
    Ok(read_log(&exit.stdout, list))
}

/// How many characters of git's words a failure keeps.
const SAID: usize = 300;

/// A failed pass in git's words (EP-42's review): its standard error alone, never its output,
/// which holds the history and the lines that hit; a line of it that holds a listed word (a ref's
/// name, say) left out; at most [`SAID`] characters.
fn failure(exit: &Exit, list: &List) -> String {
    let said = exit
        .stderr
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !list.hits(line))
        .collect::<Vec<_>>()
        .join(" ");
    let code = exit.code.map_or_else(
        || "no exit code".to_owned(),
        |code| format!("exit code {code}"),
    );
    let mut kept: String = said.chars().take(SAID).collect();
    if kept.len() < said.len() {
        kept.push('…');
    }
    if kept.is_empty() {
        format!("git log failed ({code})")
    } else {
        format!("git log failed ({code}): {kept}")
    }
}

/// A record's head: its short hash (4 to 64 hex digits), its subject and the rest; `None` for a
/// segment that is no record's start.
fn head(segment: &str) -> Option<(&str, &str, &str)> {
    let mut fields = segment.splitn(3, FIELD);
    let (short, subject, rest) = (fields.next()?, fields.next()?, fields.next()?);
    let short = short.trim();
    ((4..=64).contains(&short.len()) && short.bytes().all(|byte| byte.is_ascii_hexdigit()))
        .then_some((short, subject, rest))
}

/// The commits of `git log`'s text in [`command`]'s form and those that hit `list`. A segment
/// whose head is no record's (a NUL in a file git diffs as text) belongs to the record before it,
/// its NUL kept (EP-42's review).
pub fn read_log(text: &str, list: &List) -> Scan {
    let mut records: Vec<(&str, &str, Cow<'_, str>)> = Vec::new();
    for segment in text.split(RECORD) {
        match head(segment) {
            Some((short, subject, rest)) => records.push((short, subject, Cow::Borrowed(rest))),
            None => {
                if let Some((_, _, rest)) = records.last_mut() {
                    let rest = rest.to_mut();
                    rest.push(RECORD);
                    rest.push_str(segment);
                }
            }
        }
    }
    let mut scan = Scan {
        commits: records.len(),
        words: list.len(),
        hits: Vec::new(),
    };
    for (short, subject, rest) in &records {
        let (message, diff) = rest.split_once(MESSAGE_END).unwrap_or((rest, ""));
        let lines = message.lines().filter(|line| list.hits(line)).count()
            + read_lines(diff).filter(|line| list.hits(line)).count();
        if lines > 0 {
            let subject = subject.trim();
            scan.hits.push(Hit {
                short: (*short).to_owned(),
                subject: (!list.hits(subject)).then(|| subject.to_owned()),
                lines,
            });
        }
    }
    scan
}

/// The lines of one commit's diff the check reads: each file's `diff --git a/… b/…` header (a
/// merge's `diff --cc <path>`), so that a path counts where no `+++` line names it (a binary
/// file, an empty one, a mode change; EP-42's review); the added lines as rule 3's `grep '^+'`
/// takes them from a plain diff, the `+++ b/<path>` line included; in a merge's combined diff, a
/// line with a `+` in its parents' columns, and its `+++` line.
fn read_lines(diff: &str) -> impl Iterator<Item = &str> {
    // The parents' columns of the file's hunks: 0 in a plain diff, where a line is added when it
    // starts with `+`.
    let mut columns = 0usize;
    diff.lines().filter(move |line| {
        if line.starts_with("diff --git ") {
            columns = 0;
            return true;
        }
        if line.starts_with("diff --cc ") || line.starts_with("diff --combined ") {
            // Two parents unless a hunk's header says more.
            columns = 2;
            return true;
        }
        if columns == 0 {
            return line.starts_with('+');
        }
        if line.starts_with("@@@") {
            columns = line.bytes().take_while(|byte| *byte == b'@').count() - 1;
            return false;
        }
        if line.starts_with("+++ ") {
            return true;
        }
        line.get(..columns).is_some_and(|marks| {
            marks.contains('+') && marks.bytes().all(|mark| mark == b'+' || mark == b' ')
        })
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::actions::Real;
    use crate::files::tests::Scratch;
    use crate::revert::testing::Repo;

    #[test]
    fn a_list_reads_as_grep_takes_it_and_shows_its_count_alone() {
        let list = List::parse("\u{feff}Harbour\r\n\r\n  \nnorth gate\nÉCLAIR\r");
        assert_eq!(list.len(), 3);
        assert_eq!(format!("{list:?}"), "List(3 words)");
        assert!(list.hits("the HARBOUR notes"));
        assert!(list.hits("past the North Gate"));
        assert!(list.hits("un éclair"));
        assert!(!list.hits("northgate"), "a fixed string, spaces and all");
        assert!(
            !List::parse("\n \n").hits("anything"),
            "blank lines match nothing"
        );
        let scratch = Scratch::new("wordcheck-list");
        let file = scratch.0.join("words.txt");
        std::fs::write(&file, b"harbour\n\xff\n").unwrap();
        assert_eq!(List::read(&file).unwrap().len(), 2);
        assert!(List::named(None).is_none());
        let missing = List::named(Some(scratch.0.join("missing.txt"))).unwrap();
        assert!(missing.unwrap_err().contains("did not read"));
        // UTF-16 by its mark, as Windows PowerShell 5.1's `>` writes it, little- or big-endian.
        let text = "harbour\r\nNorth Gate\r\n";
        let little: Vec<u8> = [0xFF, 0xFE]
            .into_iter()
            .chain(text.encode_utf16().flat_map(u16::to_le_bytes))
            .collect();
        std::fs::write(&file, &little).unwrap();
        let read = List::read(&file).unwrap();
        assert_eq!(read.len(), 2);
        assert!(read.hits("past the north gate"));
        let big: Vec<u8> = [0xFE, 0xFF]
            .into_iter()
            .chain(text.encode_utf16().flat_map(u16::to_be_bytes))
            .collect();
        std::fs::write(&file, &big).unwrap();
        assert!(List::read(&file).unwrap().hits("THE HARBOUR"));
        // NUL bytes with no mark: refused, so the check says the list did not read.
        let bare: Vec<u8> = text.encode_utf16().flat_map(u16::to_le_bytes).collect();
        std::fs::write(&file, &bare).unwrap();
        assert!(List::read(&file).unwrap_err().contains("NUL bytes"));
    }

    /// A failure in git's errors alone: never its output, which holds the history, nor a line of
    /// its errors that holds a word, and cut short (EP-42's review).
    #[test]
    fn a_failure_never_quotes_the_history_or_a_word() {
        let list = List::parse("harbour\n");
        let exit = |stderr: &str| Exit {
            code: Some(128),
            stdout: "\0a1b2c3d\u{1f}x\u{1f}x\n\u{1e}\n+the harbour plan\n".to_owned(),
            stderr: stderr.to_owned(),
        };
        assert_eq!(failure(&exit(""), &list), "git log failed (exit code 128)");
        assert_eq!(
            failure(
                &exit("fatal: bad object\nwarning: ignoring broken ref refs/heads/harbour\n"),
                &list
            ),
            "git log failed (exit code 128): fatal: bad object"
        );
        let long = failure(&exit(&"e".repeat(1000)), &list);
        assert!(
            long.ends_with('…') && long.chars().count() < SAID + 40,
            "{long}"
        );
    }

    /// A synthetic log in `command`'s form: a message hit, a path hit in a header alone (a binary
    /// file, an empty one), a deleted file, a merge's combined diff, a NUL inside a text diff, and
    /// a commit that only removes a word.
    fn log() -> String {
        let record = |short: &str, subject: &str, body: &str, diff: &str| {
            format!("\0{short}\u{1f}{subject}\u{1f}{subject}\n\n{body}\u{1e}\n{diff}")
        };
        [
            record(
                "e5f6a7b",
                "Merge the side branch",
                "",
                "diff --cc notes.md\nindex 1,2..3\n--- a/notes.md\n+++ b/notes.md\n\
                 @@@ -1,1 -1,1 +1,2 @@@\n- left\n -right\n++resolved near the harbour\n \
                 +kept from the right\n",
            ),
            record(
                "d4e5f6a",
                "Remove the old note",
                "",
                "diff --git a/old.md b/old.md\ndeleted file mode 100644\nindex 1..0\n\
                 --- a/old.md\n+++ /dev/null\n@@ -1 +0,0 @@\n-the harbour plan\n",
            ),
            record(
                "c3d4e5f",
                "Add the logo",
                "",
                "diff --git a/art/harbour.png b/art/harbour.png\nnew file mode 100644\n\
                 index 0..1\nBinary files /dev/null and b/art/harbour.png differ\n",
            ),
            record(
                "f7a8b9c",
                "Keep the folder",
                "",
                "diff --git a/harbour/.gitkeep b/harbour/.gitkeep\nnew file mode 100644\n\
                 index 0000000..e69de29\n",
            ),
            record(
                "0a1b2c3",
                "Add the data",
                "",
                "diff --git a/data.txt b/data.txt\nnew file mode 100644\n--- /dev/null\n\
                 +++ b/data.txt\n@@ -0,0 +1,2 @@\n+row one\0x\u{1f}y\u{1f}z harbour\n+row two\n",
            ),
            record(
                "b2c3d4e",
                "Add the harbour notes",
                "Written from the North Gate survey.\n",
                "diff --git a/harbour/notes.md b/harbour/notes.md\nnew file mode 100644\n\
                 --- /dev/null\n+++ b/harbour/notes.md\n@@ -0,0 +1,2 @@\n+# Notes\n+plain words\n",
            ),
            record(
                "a1b2c3d",
                "Start",
                "",
                "diff --git a/README.md b/README.md\nnew file mode 100644\n--- /dev/null\n\
                 +++ b/README.md\n@@ -0,0 +1 @@\n+# A project\n@@ -3,0 +4 @@ fn harbour()\n",
            ),
        ]
        .concat()
    }

    #[test]
    fn the_log_reads_messages_paths_and_added_lines_and_masks_a_subject() {
        let list = List::parse("harbour\nnorth gate\n");
        let scan = read_log(&log(), &list);
        // The NUL inside the data's diff starts no record of its own.
        assert_eq!(scan.commits, 7);
        assert_eq!(scan.words, 2);
        let hit = |short: &str, subject: Option<&str>, lines| Hit {
            short: short.to_owned(),
            subject: subject.map(str::to_owned),
            lines,
        };
        assert_eq!(
            scan.hits,
            [
                hit("e5f6a7b", Some("Merge the side branch"), 1),
                // The paths of a binary file and an empty one, in their headers alone.
                hit("c3d4e5f", Some("Add the logo"), 1),
                hit("f7a8b9c", Some("Keep the folder"), 1),
                // The added line that holds the NUL, read whole.
                hit("0a1b2c3", Some("Add the data"), 1),
                // The message's subject and body lines (`%B`), the header and the `+++` line.
                hit("b2c3d4e", None, 4),
            ]
        );
        assert_eq!(scan.hits[4].words(), format!("b2c3d4e {MASKED} (4 lines)"));
        assert_eq!(
            scan.summary(),
            "7 commits read against a list of 2 words: 5 hit"
        );
        // Nothing hits a list of other words, and an empty log reads as no commit.
        assert!(read_log(&log(), &List::parse("lighthouse")).hits.is_empty());
        assert_eq!(read_log("", &list).commits, 0);
    }

    /// The pass over a scratch repository with real git: a word added, then removed (which does
    /// not hit), a merge whose resolution adds one, and a subject that holds one.
    #[test]
    fn a_pass_over_a_scratch_history_names_the_commits_that_hit() {
        let repo = Repo::new("wordcheck-history");
        repo.write("notes.md", "first\n");
        let start = repo.commit("Start");
        repo.write("notes.md", "first\nthe harbour plan\n");
        let added = repo.commit("Add a plan");
        repo.write("notes.md", "first\n");
        repo.commit("Take the plan out");
        repo.git(&["checkout", "--quiet", "-b", "side", &start]);
        repo.write("notes.md", "first\nfrom the side\n");
        repo.commit("Side work");
        repo.git(&["checkout", "--quiet", "main"]);
        repo.write("other.md", "other\n");
        repo.commit("Other work");
        // The merge conflicts nowhere, and its resolution adds a word of its own.
        let merge = [
            "-c",
            "user.name=Test",
            "-c",
            "user.email=test@example.invalid",
            "merge",
            "--quiet",
            "--no-commit",
            "--no-ff",
            "side",
        ];
        repo.git(&merge);
        repo.write("notes.md", "first\nfrom the side\nnear the HARBOUR\n");
        let merged = repo.commit("Merge the side");
        repo.write("readme.md", "plain\n");
        let subject = repo.commit("Harbour readme");
        let list = List::parse("harbour\n");
        let scan = history(&repo.0, &list, &Real).unwrap();
        assert_eq!(scan.commits, 7);
        let short = |full: &str| full[..7].to_owned();
        let hits: Vec<(String, Option<String>)> = scan
            .hits
            .iter()
            .map(|hit| (hit.short[..7].to_owned(), hit.subject.clone()))
            .collect();
        assert_eq!(
            hits,
            [
                (short(&subject), None),
                (short(&merged), Some("Merge the side".to_owned())),
                (short(&added), Some("Add a plan".to_owned())),
            ],
            "{:?}",
            scan.hits
        );
        // A folder that is no repository: git's own words.
        let scratch = Scratch::new("wordcheck-none");
        assert!(history(&scratch.0, &list, &Real).is_err());
    }
}
