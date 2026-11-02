//! The owner's VS Code workspace files (D-7): the aggregate, which lists every folder, and each
//! project's own single-folder file beside its folder (the D-7 addendum). For the aggregate:
//! which folders it lists, and one entry appended without touching any other byte; the file is
//! JSON with comments and trailing commas (JSONC), so a small scanner, not a JSON parser, finds
//! the `folders` array, its entries and their layout. For the own file: where it goes and its
//! text in the owner's form.

use std::fs;
use std::path::{Path, PathBuf};

use crate::paths;

/// Where a folder's own workspace file goes: `<name>.code-workspace` beside it; `None` for a
/// path without a name or a parent.
pub fn own_file(folder: &Path) -> Option<PathBuf> {
    let mut name = folder.file_name()?.to_owned();
    name.push(".code-workspace");
    Some(folder.parent()?.join(name))
}

/// The text of a folder's own workspace file in the owner's form: the one folder under its own
/// name, beside the file, and empty settings; tabs, LF and a final newline.
pub fn single_folder(name: &str) -> String {
    let name = serde_json::to_string(name).expect("a string serializes");
    format!(
        "{{\n\t\"folders\": [\n\t\t{{\n\t\t\t\"name\": {name},\n\t\t\t\"path\": {name}\n\t\t}}\n\t],\n\t\"settings\": {{}}\n}}\n"
    )
}

/// The `.code-workspace` files of `dir` that list more than one folder, by name in name order:
/// the candidates for the aggregate a fresh registry names (D-7 addendum). A file that does not
/// read is passed over.
pub fn aggregates(dir: &Path) -> Vec<String> {
    let lists_several = |path: &Path| {
        fs::read_to_string(path)
            .ok()
            .and_then(|text| entries(&text).ok())
            .is_some_and(|found| found.len() > 1)
    };
    let mut found: Vec<String> = fs::read_dir(dir)
        .into_iter()
        .flatten()
        .filter_map(Result::ok)
        .filter(|entry| {
            entry
                .file_name()
                .to_string_lossy()
                .to_lowercase()
                .ends_with(".code-workspace")
                && entry.path().is_file()
                && lists_several(&entry.path())
        })
        .map(|entry| entry.file_name().to_string_lossy().into_owned())
        .collect();
    found.sort_by_key(|name| name.to_lowercase());
    found
}

/// One entry of the `folders` array.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Entry {
    pub path: Option<String>,
    pub name: Option<String>,
}

/// The entries of the file's `folders` array.
pub fn entries(text: &str) -> Result<Vec<Entry>, String> {
    Ok(scan(text)?
        .elements
        .into_iter()
        .map(|element| element.entry)
        .collect())
}

/// Whether one of `entries` is `folder`; relative entries are read against `dir`, the folder that
/// holds the workspace file, as VS Code reads them.
pub fn lists(entries: &[Entry], dir: &Path, folder: &Path) -> bool {
    entries
        .iter()
        .filter_map(|entry| entry.path.as_deref())
        .any(|path| paths::same(&dir.join(path), folder))
}

/// The `path` a new entry for `folder` takes: its name when it sits beside the workspace file,
/// else its full path with forward slashes.
pub fn entry_path(dir: &Path, folder: &Path) -> String {
    match (folder.parent(), folder.file_name()) {
        (Some(parent), Some(name)) if paths::same(parent, dir) => {
            name.to_string_lossy().into_owned()
        }
        _ => folder.to_string_lossy().replace('\\', "/"),
    }
}

/// `text` with `{ "path": <path> }` appended to `folders`, laid out like the entry before it:
/// the same line break and indentation before it, inside it and before its closing brace.
pub fn append(text: &str, path: &str) -> Result<String, String> {
    let scan = scan(text)?;
    let value = serde_json::to_string(path).expect("a string serializes");
    let (at, insert) = match scan.elements.last() {
        Some(last) => {
            let entry = match &last.layout {
                Some(layout) => {
                    let piece = |(from, to): (usize, usize)| &text[from..to];
                    // A comment between the key and the value is not copied.
                    let colon = layout
                        .colon
                        .map(piece)
                        .filter(|between| !between.contains('/'))
                        .unwrap_or(": ");
                    format!(
                        "{{{}\"path\"{colon}{value}{}}}",
                        indent(piece(layout.inner)),
                        indent(piece(layout.close))
                    )
                }
                None => format!("{{ \"path\": {value} }}"),
            };
            (
                last.end,
                format!(",{}{entry}", indent(&text[last.lead..last.start])),
            )
        }
        None => {
            // An empty list: the entry on its own line one tab deeper than the closing bracket,
            // or inline when the brackets share a line.
            let inside = indent(&text[scan.open + 1..scan.close]);
            let lead = if inside.contains('\n') {
                format!("{inside}\t")
            } else {
                String::new()
            };
            (scan.open + 1, format!("{lead}{{ \"path\": {value} }}"))
        }
    };
    let mut out = String::with_capacity(text.len() + insert.len());
    out.push_str(&text[..at]);
    out.push_str(&insert);
    out.push_str(&text[at..]);
    let again = entries(&out)?;
    if again.len() != scan.elements.len() + 1
        || again.last().and_then(|entry| entry.path.as_deref()) != Some(path)
    {
        return Err("the new entry did not read back".to_owned());
    }
    Ok(out)
}

/// The layout part of the text between two tokens: from its last line break (with a `\r` before
/// it) through that line's indentation; on one line, its leading spaces.
fn indent(between: &str) -> &str {
    let blank = |s: &str| s.len() - s.trim_start_matches([' ', '\t']).len();
    match between.rfind('\n') {
        Some(nl) => {
            let start = if between[..nl].ends_with('\r') {
                nl - 1
            } else {
                nl
            };
            &between[start..nl + 1 + blank(&between[nl + 1..])]
        }
        None => &between[..blank(between)],
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Kind {
    Open(u8),
    Close(u8),
    Comma,
    Colon,
    Str,
    /// A number, `true`, `false` or `null`.
    Word,
}

#[derive(Debug, Clone, Copy)]
struct Token {
    kind: Kind,
    start: usize,
    end: usize,
}

/// The tokens of `text` with their byte ranges; whitespace and both comment forms are skipped.
fn tokenize(text: &str) -> Result<Vec<Token>, String> {
    let bytes = text.as_bytes();
    let mut tokens = Vec::new();
    let mut i = if text.starts_with('\u{feff}') { 3 } else { 0 };
    while i < bytes.len() {
        let start = i;
        let kind = match bytes[i] {
            b' ' | b'\t' | b'\r' | b'\n' => {
                i += 1;
                continue;
            }
            b'/' if bytes.get(i + 1) == Some(&b'/') => {
                while i < bytes.len() && bytes[i] != b'\n' {
                    i += 1;
                }
                continue;
            }
            b'/' if bytes.get(i + 1) == Some(&b'*') => {
                let close = text[i + 2..].find("*/").ok_or("a comment is not closed")?;
                i += 2 + close + 2;
                continue;
            }
            b'"' => {
                i += 1;
                loop {
                    match bytes.get(i) {
                        None => return Err("a string is not closed".to_owned()),
                        Some(b'\\') => i += 2,
                        Some(b'"') => break i += 1,
                        Some(_) => i += 1,
                    }
                }
                Kind::Str
            }
            c @ (b'{' | b'[') => {
                i += 1;
                Kind::Open(c)
            }
            c @ (b'}' | b']') => {
                i += 1;
                Kind::Close(c)
            }
            b',' => {
                i += 1;
                Kind::Comma
            }
            b':' => {
                i += 1;
                Kind::Colon
            }
            _ => {
                i += 1;
                while i < bytes.len() && !b" \t\r\n{}[],:\"/".contains(&bytes[i]) {
                    i += 1;
                }
                Kind::Word
            }
        };
        tokens.push(Token {
            kind,
            start,
            end: i,
        });
    }
    Ok(tokens)
}

/// The `folders` array: where its brackets are, and its elements.
struct Scan {
    open: usize,
    close: usize,
    elements: Vec<Element>,
}

struct Element {
    /// Just after the `[` or the `,` before it.
    lead: usize,
    start: usize,
    end: usize,
    entry: Entry,
    /// For an object, the pieces of its layout a new entry copies.
    layout: Option<Layout>,
}

/// Byte ranges inside an object entry.
struct Layout {
    /// From just after `{` to its first key.
    inner: (usize, usize),
    /// From the end of the first key to the start of its value.
    colon: Option<(usize, usize)>,
    /// From the end of the last member (or its trailing comma) to `}`.
    close: (usize, usize),
}

struct Member {
    key: String,
    key_token: Token,
    /// The index of the value's first token.
    value_at: usize,
}

struct Object {
    open: Token,
    close: Token,
    members: Vec<Member>,
    after_last: usize,
}

struct Parser<'a> {
    text: &'a str,
    tokens: &'a [Token],
    pos: usize,
}

fn scan(text: &str) -> Result<Scan, String> {
    let tokens = tokenize(text)?;
    let mut parser = Parser {
        text,
        tokens: &tokens,
        pos: 0,
    };
    let root = parser.object()?;
    let mut folders = root.members.iter().filter(|member| member.key == "folders");
    let member = folders.next().ok_or("the file has no folders list")?;
    if folders.next().is_some() {
        return Err("the file has two folders lists".to_owned());
    }
    parser.pos = member.value_at;
    parser.array()
}

impl Parser<'_> {
    fn next(&mut self) -> Result<Token, String> {
        let token = *self.tokens.get(self.pos).ok_or("the file ends too early")?;
        self.pos += 1;
        Ok(token)
    }

    fn unexpected(&self, token: Token, wanted: &str) -> String {
        format!("expected {wanted} at byte {}", token.start)
    }

    fn string(&self, token: Token) -> Result<String, String> {
        serde_json::from_str(&self.text[token.start..token.end])
            .map_err(|e| format!("a string at byte {}: {e}", token.start))
    }

    /// Skips one value of any kind and returns its last token.
    fn skip_value(&mut self) -> Result<Token, String> {
        let first = self.next()?;
        let mut open = match first.kind {
            Kind::Str | Kind::Word => return Ok(first),
            Kind::Open(c) => vec![c],
            _ => return Err(self.unexpected(first, "a value")),
        };
        loop {
            let token = self.next()?;
            match token.kind {
                Kind::Open(c) => open.push(c),
                Kind::Close(c) => {
                    let pair = (open.pop().unwrap_or(0), c);
                    if pair != (b'{', b'}') && pair != (b'[', b']') {
                        return Err(self.unexpected(token, "a matching bracket"));
                    }
                    if open.is_empty() {
                        return Ok(token);
                    }
                }
                _ => {}
            }
        }
    }

    /// The object whose `{` is the next token, with each member's key and where its value
    /// starts; the values are skipped.
    fn object(&mut self) -> Result<Object, String> {
        let open = self.next()?;
        if open.kind != Kind::Open(b'{') {
            return Err(self.unexpected(open, "an object"));
        }
        let mut members = Vec::new();
        let mut after_last = open.end;
        loop {
            let token = self.next()?;
            match token.kind {
                Kind::Close(b'}') => {
                    break Ok(Object {
                        open,
                        close: token,
                        members,
                        after_last,
                    });
                }
                Kind::Str => {
                    let key = self.string(token)?;
                    let colon = self.next()?;
                    if colon.kind != Kind::Colon {
                        return Err(self.unexpected(colon, "a colon"));
                    }
                    let value_at = self.pos;
                    after_last = self.skip_value()?.end;
                    members.push(Member {
                        key,
                        key_token: token,
                        value_at,
                    });
                    let token = self.next()?;
                    match token.kind {
                        // A trailing comma is allowed: the loop then meets the brace.
                        Kind::Comma => after_last = token.end,
                        Kind::Close(b'}') => {
                            break Ok(Object {
                                open,
                                close: token,
                                members,
                                after_last,
                            });
                        }
                        _ => return Err(self.unexpected(token, "a comma or a closing brace")),
                    }
                }
                _ => return Err(self.unexpected(token, "a key")),
            }
        }
    }

    /// The `folders` array, whose `[` is the next token.
    fn array(&mut self) -> Result<Scan, String> {
        let open = self.next()?;
        if open.kind != Kind::Open(b'[') {
            return Err("folders is not a list".to_owned());
        }
        let mut elements = Vec::new();
        let mut lead = open.end;
        loop {
            let first = *self.tokens.get(self.pos).ok_or("the file ends too early")?;
            if first.kind == Kind::Close(b']') {
                self.pos += 1;
                break Ok(Scan {
                    open: open.start,
                    close: first.start,
                    elements,
                });
            }
            elements.push(if first.kind == Kind::Open(b'{') {
                self.entry(lead)?
            } else {
                let last = self.skip_value()?;
                Element {
                    lead,
                    start: first.start,
                    end: last.end,
                    entry: Entry::default(),
                    layout: None,
                }
            });
            let token = self.next()?;
            match token.kind {
                // A trailing comma is allowed: the loop then meets the bracket.
                Kind::Comma => lead = token.end,
                Kind::Close(b']') => {
                    break Ok(Scan {
                        open: open.start,
                        close: token.start,
                        elements,
                    });
                }
                _ => return Err(self.unexpected(token, "a comma or a closing bracket")),
            }
        }
    }

    /// An object entry of `folders`, with its `path` and `name` when they are strings.
    fn entry(&mut self, lead: usize) -> Result<Element, String> {
        let object = self.object()?;
        let field = |key: &str| {
            object
                .members
                .iter()
                .rev()
                .find(|member| member.key == key)
                .map(|member| self.tokens[member.value_at])
                .filter(|token| token.kind == Kind::Str)
                .and_then(|token| self.string(token).ok())
        };
        let first = object.members.first();
        Ok(Element {
            lead,
            start: object.open.start,
            end: object.close.end,
            entry: Entry {
                path: field("path"),
                name: field("name"),
            },
            layout: Some(Layout {
                inner: (
                    object.open.end,
                    first.map_or(object.close.start, |member| member.key_token.start),
                ),
                colon: first
                    .map(|member| (member.key_token.end, self.tokens[member.value_at].start)),
                close: (object.after_last, object.close.start),
            }),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::files::tests::Scratch;

    #[test]
    fn an_own_file_sits_beside_its_folder_in_the_owners_form() {
        assert_eq!(
            own_file(Path::new(r"C:\Data\alpha")),
            Some(PathBuf::from(r"C:\Data\alpha.code-workspace"))
        );
        assert_eq!(own_file(Path::new(r"C:\")), None);
        // The owner's form, as their own files are written.
        let text = single_folder("alpha");
        assert_eq!(
            text,
            "{\n\t\"folders\": [\n\t\t{\n\t\t\t\"name\": \"alpha\",\n\t\t\t\"path\": \"alpha\"\n\t\t}\n\t],\n\t\"settings\": {}\n}\n"
        );
        assert_eq!(
            entries(&text).unwrap(),
            [Entry {
                path: Some("alpha".to_owned()),
                name: Some("alpha".to_owned()),
            }]
        );
        let odd = single_folder(r#"odd "name""#);
        assert_eq!(
            entries(&odd).unwrap()[0].path.as_deref(),
            Some(r#"odd "name""#)
        );
        assert!(lists(
            &entries(&text).unwrap(),
            Path::new(r"C:\Data"),
            Path::new(r"C:\Data\Alpha")
        ));
    }

    #[test]
    fn the_aggregate_is_a_file_that_lists_several_folders() {
        let scratch = Scratch::new("workspace-aggregates");
        let dir = &scratch.0;
        assert!(aggregates(dir).is_empty());
        for name in ["alpha", "beta"] {
            fs::write(
                dir.join(format!("{name}.code-workspace")),
                single_folder(name),
            )
            .unwrap();
        }
        fs::write(dir.join("broken.code-workspace"), "{ \"folders\": [").unwrap();
        fs::create_dir(dir.join("folder.code-workspace")).unwrap();
        assert!(aggregates(dir).is_empty(), "single-folder files only");
        fs::write(dir.join("All.code-workspace"), VS_CODE).unwrap();
        assert_eq!(aggregates(dir), ["All.code-workspace"]);
        fs::write(dir.join("more.CODE-WORKSPACE"), VS_CODE).unwrap();
        assert_eq!(
            aggregates(dir),
            ["All.code-workspace", "more.CODE-WORKSPACE"]
        );
        assert!(aggregates(&dir.join("missing")).is_empty());
    }

    /// The layout VS Code writes, as in the owner's file: tabs, LF, no final newline, older
    /// entries with a name, newer ones without.
    const VS_CODE: &str = "{\n\t\"folders\": [\n\t\t{\n\t\t\t\"name\": \"alpha\",\n\t\t\t\"path\": \"alpha\"\n\t\t},\n\t\t{\n\t\t\t\"name\": \"Vault\",\n\t\t\t\"path\": \"../Vault\"\n\t\t},\n\t\t{\n\t\t\t\"path\": \"beta\"\n\t\t}\n\t]\n}";

    #[test]
    fn reads_the_entries() {
        let found = entries(VS_CODE).unwrap();
        let paths: Vec<_> = found.iter().map(|e| e.path.as_deref().unwrap()).collect();
        assert_eq!(paths, ["alpha", "../Vault", "beta"]);
        assert_eq!(found[1].name.as_deref(), Some("Vault"));
        assert_eq!(found[2].name, None);
    }

    #[test]
    fn appends_after_the_last_entry_in_its_layout() {
        let out = append(VS_CODE, "gamma").unwrap();
        let expected = VS_CODE.replace(
            "\"beta\"\n\t\t}\n",
            "\"beta\"\n\t\t},\n\t\t{\n\t\t\t\"path\": \"gamma\"\n\t\t}\n",
        );
        assert_eq!(out, expected);
        // Every byte before and after the new entry is the original's.
        let at = VS_CODE.rfind("\n\t]").unwrap();
        assert_eq!(&out[..at], &VS_CODE[..at]);
        assert!(out.ends_with(&VS_CODE[at..]));
    }

    #[test]
    fn copies_inline_layouts_and_keeps_comments_trailing_commas_and_crlf() {
        let inline = r#"{"folders": [{ "path": "a" }, { "path": "b" }], "settings": {}}"#;
        assert_eq!(
            append(inline, "c").unwrap(),
            r#"{"folders": [{ "path": "a" }, { "path": "b" }, { "path": "c" }], "settings": {}}"#
        );
        let tight = r#"{"folders":[{"path":"a"}]}"#;
        assert_eq!(
            append(tight, "b").unwrap(),
            r#"{"folders":[{"path":"a"},{"path":"b"}]}"#
        );

        let commented = "{\r\n  // the folders [not these]\r\n  \"folders\": [\r\n    { \"path\": \"a\" }, // first\r\n    /* second */ { \"name\": \"B\", \"path\": \"b\" },\r\n  ],\r\n  \"settings\": { \"x\": \"[not a folder]\" },\r\n}\r\n";
        let out = append(commented, "c").unwrap();
        assert_eq!(
            out,
            commented.replace(
                "\"path\": \"b\" },\r\n",
                "\"path\": \"b\" },\r\n    { \"path\": \"c\" },\r\n"
            )
        );
        let paths: Vec<_> = entries(&out)
            .unwrap()
            .into_iter()
            .map(|e| e.path.unwrap())
            .collect();
        assert_eq!(paths, ["a", "b", "c"]);
    }

    #[test]
    fn fills_an_empty_list() {
        assert_eq!(
            append("{\n\t\"folders\": [\n\t]\n}", "a").unwrap(),
            "{\n\t\"folders\": [\n\t\t{ \"path\": \"a\" }\n\t]\n}"
        );
        assert_eq!(
            append(r#"{"folders": []}"#, "a").unwrap(),
            r#"{"folders": [{ "path": "a" }]}"#
        );
    }

    #[test]
    fn escapes_the_path() {
        let out = append(r#"{"folders": []}"#, r#"C:/odd "name""#).unwrap();
        assert_eq!(out, r#"{"folders": [{ "path": "C:/odd \"name\"" }]}"#);
        assert_eq!(
            entries(&out).unwrap()[0].path.as_deref(),
            Some(r#"C:/odd "name""#)
        );
    }

    #[test]
    fn refuses_what_it_cannot_read() {
        for (text, message) in [
            ("[]", "an object"),
            (r#"{"settings": {}}"#, "no folders list"),
            (r#"{"folders": {}}"#, "not a list"),
            (r#"{"folders": [], "folders": []}"#, "two folders lists"),
            (r#"{"folders": [{"path": "a}]}"#, "not closed"),
            ("{\"folders\": [] /* open", "not closed"),
            (r#"{"folders": [{"path": "a"}"#, "ends too early"),
            (r#"{"folders": [{"path" "a"}]}"#, "a colon"),
            (r#"{"folders": [{"path": "a"]]}"#, "matching bracket"),
        ] {
            let error = append(text, "x").unwrap_err();
            assert!(error.contains(message), "{text}: {error}");
        }
    }

    #[test]
    fn knows_which_folders_are_listed() {
        let dir = Path::new(r"C:\Data");
        let found = entries(
            r#"{"folders": [{"path": "alpha"}, {"path": "./beta/"}, {"path": "C:/Data/GAMMA"},
               {"path": "../Vault"}, {"name": "no path"}, {"uri": "vscode-remote://x"}]}"#,
        )
        .unwrap();
        for (folder, listed) in [
            (r"C:\Data\alpha", true),
            (r"C:\Data\beta", true),
            (r"C:\Data\gamma", true),
            (r"C:\Vault", true),
            (r"C:\Data\delta", false),
            (r"C:\Data\alphabet", false),
        ] {
            assert_eq!(lists(&found, dir, Path::new(folder)), listed, "{folder}");
        }
        assert_eq!(entry_path(dir, &PathBuf::from(r"c:\data\delta")), "delta");
        assert_eq!(
            entry_path(dir, Path::new(r"D:\Other\epsilon")),
            "D:/Other/epsilon"
        );
    }
}
