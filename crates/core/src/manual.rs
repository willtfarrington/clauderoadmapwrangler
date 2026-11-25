//! A project's manual as PDFs (D-12 addendum of 2026-11-25; EP-57): `docs/manual.md`, or
//! `docs/manual-changes.md`, turned into Typst by the tool itself ([`convert`]) inside one
//! template ([`document`]) and built by the Typst CLI, pinned at [`TYPST`], into the local
//! folder's `manuals` ([`build`]). Every run of text becomes a Typst string literal and the
//! structure becomes calls, so no character of the Markdown needs Typst's markup escapes. The CLI
//! reads the generated source from its standard input with `--root` at the project's `docs`, so
//! nothing is written into the project and nothing outside its `docs` is read; with Typst's own
//! fonts alone, no package and a creation timestamp from the source's last commit, a build is
//! offline and the same source gives the same bytes.

use std::ffi::OsString;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use chrono::{DateTime, Local};
use pulldown_cmark::{Alignment, CodeBlockKind, Event, Options, Parser, Tag, TagEnd};

use crate::process::{self, Exit};
use crate::version::Version;

/// The Typst CLI the template is checked under (D-12 addendum): exactly this version, since a
/// newer one may read the template otherwise and changes the bytes. It moves only in a brief that
/// needs a newer Typst and checks the template under it.
pub const TYPST: Version = Version::new(0, 15, 1);

/// How Typst is installed at the pinned version: winget's portable package, no administrator.
pub const INSTALL: &str = "winget install --id Typst.Typst --version 0.15.1";

/// How it is held there against `winget upgrade --all`.
pub const PIN: &str = "winget pin add --id Typst.Typst --version 0.15.1";

/// The folder of a project that holds the manual's files and its pictures: Typst's root.
pub const DOCS: &str = "docs";

/// The folder under the local folder that receives the PDFs.
pub const MANUALS: &str = "manuals";

/// A build's limit: Typst builds a manual in well under a second.
const COMPILE_TIMEOUT: Duration = Duration::from_secs(120);

/// Variables that would change what Typst reads or writes, taken out of its environment: fonts
/// from elsewhere, another root, another timestamp, packages.
const TYPST_ENV: [&str; 6] = [
    "TYPST_FONT_PATHS",
    "TYPST_ROOT",
    "TYPST_IGNORE_EMBEDDED_FONTS",
    "TYPST_PACKAGE_PATH",
    "TYPST_PACKAGE_CACHE_PATH",
    "SOURCE_DATE_EPOCH",
];

/// The manuals folder: `%LOCALAPPDATA%\clauderoadmapwrangler\manuals`, or `CRW_HOME`'s.
pub fn dir() -> Option<PathBuf> {
    Some(crate::paths::local_dir()?.join(MANUALS))
}

/// Which of a project's two files are there, read with its other files (two looks, no child;
/// EP-57a): the window greys its Manual entries by it and dates the manual in the Status section.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Present {
    /// `docs/manual.md`'s own time in seconds since 1970, when it is there: a project without it
    /// owes no manual (D-12 addendum).
    pub manual: Option<i64>,
    /// `docs/manual-changes.md` is there.
    pub changes: bool,
}

impl Present {
    pub fn read(folder: &Path) -> Self {
        let docs = folder.join(DOCS);
        let manual = docs.join(Which::Manual.file());
        Self {
            manual: manual
                .is_file()
                .then(|| modified(&manual).map_or(0, seconds)),
            changes: docs.join(Which::Changes.file()).is_file(),
        }
    }

    /// Whether `which`'s PDF can be built from the files that are there.
    pub fn has(self, which: Which) -> bool {
        match which {
            Which::Manual => self.manual.is_some(),
            Which::Changes => self.manual.is_some() && self.changes,
        }
    }
}

/// Opens `pdf` with the system's viewer for its kind (EP-57a), in the children's turn: Explorer
/// hands a file to the program its type opens with and exits, so nothing is waited for. The path
/// goes with `\` alone (a `CRW_HOME` may be written with `/`, which Explorer reads as a switch)
/// and whole in quotes (it splits at commas).
pub fn open(pdf: &Path) -> Result<(), String> {
    let windows = std::env::var_os("SystemRoot")
        .filter(|v| !v.is_empty())
        .map_or_else(|| PathBuf::from(r"C:\Windows"), PathBuf::from);
    let explorer = windows.join("explorer.exe");
    let pdf = normalised(pdf);
    let _turn = process::turn();
    process::start_quoted(&explorer, pdf.as_os_str()).map(drop)
}

/// `path` rebuilt from its parts, so that its separators are the system's own.
pub fn normalised(path: &Path) -> PathBuf {
    path.components().collect()
}

/// Which of a project's two files a PDF is built from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Which {
    /// `docs/manual.md`.
    Manual,
    /// `docs/manual-changes.md`: what changed since the last phase's close.
    Changes,
}

impl Which {
    /// The file's name in `docs`.
    pub fn file(self) -> &'static str {
        match self {
            Which::Manual => "manual.md",
            Which::Changes => "manual-changes.md",
        }
    }

    /// The file's path from the project's folder, with `/`: `docs/manual.md`.
    pub fn source(self) -> String {
        format!("{DOCS}/{}", self.file())
    }

    /// The PDF's name: `<project>-manual.pdf` or `<project>-manual-changes.pdf`.
    pub fn pdf(self, project: &str) -> String {
        match self {
            Which::Manual => format!("{project}-manual.pdf"),
            Which::Changes => format!("{project}-manual-changes.pdf"),
        }
    }

    /// The title when the file has no `#` heading.
    fn fallback_title(self, project: &str) -> String {
        match self {
            Which::Manual => format!("{project} — manual"),
            Which::Changes => format!("{project} — changes since the last phase's close"),
        }
    }
}

/// The Markdown as Typst, in parts the template places.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Converted {
    /// The first `#` heading's text, plain: the title page's title.
    pub title: Option<String>,
    /// What comes before the first chapter, as Typst: the title page's text, which names the
    /// phase the manual describes.
    pub intro: String,
    /// The chapters, as Typst.
    pub body: String,
    /// The pictures the Markdown names, from `docs` with `/`, in order, each once.
    pub pictures: Vec<String>,
    /// What was left out or read otherwise, each with its line.
    pub warnings: Vec<String>,
}

/// A Typst string literal holding `text` as it is: only `\` and `"` escaped, a tab and a line
/// break as their escapes, a carriage return dropped (so a checkout's line endings change no
/// byte) and other control characters by their code.
pub fn literal(text: &str) -> String {
    let mut out = String::with_capacity(text.len() + 2);
    out.push('"');
    for c in text.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\t' => out.push_str("\\t"),
            '\r' => {}
            c if c.is_control() => out.push_str(&format!("\\u{{{:x}}}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

/// What a frame of the converter's stack collects until its end.
enum Frame {
    /// A paragraph: the figures of the pictures it holds, at any depth, and whether it holds
    /// anything else, so that pictures alone in their paragraph become figures at block level.
    Paragraph {
        figures: Vec<String>,
        other: bool,
    },
    /// A heading of the Markdown's level; the first `#` is the title.
    Heading {
        level: usize,
        title: bool,
    },
    Quote,
    Code {
        lang: Option<String>,
        text: String,
    },
    List {
        start: Option<u64>,
        items: Vec<String>,
    },
    Item,
    Table {
        align: Vec<Alignment>,
        header: Vec<String>,
        cells: Vec<String>,
    },
    Head,
    Row,
    Cell,
    Emphasis,
    Strong,
    Strike,
    /// A link: its address when it has a scheme, else its text alone.
    Link {
        url: Option<String>,
    },
    /// A picture: its path from `docs`, `None` when it is left out; its alt text is the caption.
    Picture {
        path: Option<String>,
    },
    /// Raw HTML, left out: its text, to tell a comment from markup.
    Html {
        text: String,
        line: usize,
    },
    /// Anything else: its content kept as it is.
    Plain,
}

struct Open {
    frame: Frame,
    /// The frame's content as Typst.
    out: String,
    /// Its text, plain: a heading's, for the title.
    plain: String,
}

struct Converter<'a> {
    source: &'a str,
    stack: Vec<Open>,
    converted: Converted,
    /// Whether a chapter has begun: block content then goes to the body, before it to the intro.
    in_body: bool,
}

/// Turns a manual's Markdown into Typst, in parts: the title, the intro and the chapters, with
/// the pictures it names and the warnings for what it leaves out.
pub fn convert(markdown: &str) -> Converted {
    let markdown = markdown.strip_prefix('\u{feff}').unwrap_or(markdown);
    let mut converter = Converter {
        source: markdown,
        stack: Vec::new(),
        converted: Converted::default(),
        in_body: false,
    };
    let options = Options::ENABLE_TABLES | Options::ENABLE_STRIKETHROUGH;
    for (event, range) in Parser::new_ext(markdown, options).into_offset_iter() {
        converter.event(event, range.start);
    }
    converter.converted
}

impl Converter<'_> {
    fn line(&self, offset: usize) -> usize {
        self.source[..offset.min(self.source.len())]
            .matches('\n')
            .count()
            + 1
    }

    fn warn(&mut self, offset: usize, words: &str) {
        let line = self.line(offset);
        self.converted
            .warnings
            .push(format!("line {line}: {words}"));
    }

    fn open(&mut self, frame: Frame) {
        self.stack.push(Open {
            frame,
            out: String::new(),
            plain: String::new(),
        });
    }

    /// Inline content into the innermost frame: a paragraph's marks it as holding more than a
    /// picture.
    fn inline(&mut self, typst: &str, plain: &str) {
        match self.stack.last_mut() {
            Some(open) => {
                if let Frame::Code { text, .. } = &mut open.frame {
                    text.push_str(plain);
                    return;
                }
                if let Frame::Html { text, .. } = &mut open.frame {
                    text.push_str(plain);
                    return;
                }
                if let Frame::Paragraph { other, .. } = &mut open.frame
                    && !plain.trim().is_empty()
                {
                    *other = true;
                }
                open.out.push_str(typst);
                open.plain.push_str(plain);
            }
            // Text outside any block: a paragraph of its own.
            None => self.block(format!("#par[{typst}]")),
        }
    }

    /// A block into the innermost container, or into the intro or the body, a blank line apart.
    fn block(&mut self, typst: String) {
        match self.stack.last_mut() {
            Some(open) => {
                if let Frame::Paragraph { other, .. } = &mut open.frame {
                    *other = true;
                }
                open.out.push_str(&typst);
            }
            None => {
                let flow = if self.in_body {
                    &mut self.converted.body
                } else {
                    &mut self.converted.intro
                };
                if !flow.is_empty() {
                    flow.push_str("\n\n");
                }
                flow.push_str(&typst);
            }
        }
    }

    fn event(&mut self, event: Event, offset: usize) {
        match event {
            Event::Start(tag) => self.start(tag, offset),
            Event::End(tag) => self.end(tag, offset),
            Event::Text(text) => self.inline(&format!("#{}", literal(&text)), &text),
            Event::Code(code) => self.inline(&format!("#raw({})", literal(&code)), &code),
            Event::SoftBreak => self.inline("#\" \"", " "),
            Event::HardBreak => self.inline("#linebreak()", " "),
            Event::Rule => self.block("#line(length: 100%)".to_owned()),
            Event::Html(html) => self.inline("", &html),
            Event::InlineHtml(html) => {
                if !html.trim_start().starts_with("<!--") {
                    self.warn(offset, &format!("raw HTML left out: {}", html.trim()));
                }
            }
            // Not enabled, so not met; kept as the text they are.
            Event::InlineMath(text) | Event::DisplayMath(text) | Event::FootnoteReference(text) => {
                self.inline(&format!("#{}", literal(&text)), &text);
            }
            Event::TaskListMarker(done) => {
                let mark = if done { "[x] " } else { "[ ] " };
                self.inline(&format!("#{}", literal(mark)), mark);
            }
        }
    }

    fn start(&mut self, tag: Tag, offset: usize) {
        let frame = match tag {
            Tag::Paragraph => Frame::Paragraph {
                figures: Vec::new(),
                other: false,
            },
            Tag::Heading { level, .. } => {
                let level = level as usize;
                let title = level == 1
                    && self.converted.title.is_none()
                    && !self.in_body
                    && self.stack.is_empty();
                Frame::Heading { level, title }
            }
            Tag::BlockQuote(_) => Frame::Quote,
            Tag::CodeBlock(kind) => Frame::Code {
                lang: match kind {
                    CodeBlockKind::Fenced(info) => info
                        .split_whitespace()
                        .next()
                        .filter(|lang| !lang.is_empty())
                        .map(str::to_owned),
                    CodeBlockKind::Indented => None,
                },
                text: String::new(),
            },
            Tag::HtmlBlock => Frame::Html {
                text: String::new(),
                line: self.line(offset),
            },
            Tag::List(start) => Frame::List {
                start,
                items: Vec::new(),
            },
            Tag::Item => Frame::Item,
            Tag::Table(align) => Frame::Table {
                align,
                header: Vec::new(),
                cells: Vec::new(),
            },
            Tag::TableHead => Frame::Head,
            Tag::TableRow => Frame::Row,
            Tag::TableCell => Frame::Cell,
            Tag::Emphasis => Frame::Emphasis,
            Tag::Strong => Frame::Strong,
            Tag::Strikethrough => Frame::Strike,
            Tag::Link { dest_url, .. } => Frame::Link {
                url: has_scheme(&dest_url).then(|| dest_url.to_string()),
            },
            Tag::Image { dest_url, .. } => {
                let path = picture_path(&dest_url);
                if let Err(why) = &path {
                    self.warn(
                        offset,
                        &format!("the picture {dest_url} is left out: {why}"),
                    );
                }
                Frame::Picture { path: path.ok() }
            }
            _ => Frame::Plain,
        };
        self.open(frame);
    }

    fn end(&mut self, _tag: TagEnd, offset: usize) {
        let Some(Open { frame, out, plain }) = self.stack.pop() else {
            return;
        };
        match frame {
            // Pictures alone (a line break between them aside) are figures; amid text they stay
            // in the line, since Typst leaves a figure inside a paragraph out.
            Frame::Paragraph { figures, other } if !other && !figures.is_empty() => {
                for figure in figures {
                    self.block(figure);
                }
            }
            Frame::Paragraph { .. } => self.block(format!("#par[{out}]")),
            Frame::Heading { title: true, .. } => {
                self.converted.title = Some(plain.trim().to_owned());
            }
            Frame::Heading { level, .. } => {
                let nested = !self.stack.is_empty();
                if level == 1 {
                    let words = if self.converted.title.is_some() {
                        "a second # heading, read as a chapter: the manual's title is its first"
                    } else if nested {
                        "a # heading inside a quote or a list is not the title: the title is a # \
                         heading before the first chapter, outside any block"
                    } else {
                        "a # heading after the first chapter is not the title, read as a \
                         chapter: the title is a # heading before the first chapter"
                    };
                    self.warn(offset, words);
                }
                let level = level.saturating_sub(1).max(1);
                if nested {
                    // Inside a quote or a list a heading starts no page (Typst refuses a page
                    // break inside a container) and stays out of the contents.
                    self.block(format!(
                        "#heading(level: {level}, outlined: false, numbering: none)[{out}]"
                    ));
                } else {
                    self.in_body = true;
                    self.block(format!("#heading(level: {level})[{out}]"));
                }
                if let Some(parent) = self.stack.last_mut() {
                    parent.plain.push_str(&plain);
                }
            }
            Frame::Quote => self.block(format!("#quote(block: true)[{out}]")),
            Frame::Code { lang, text } => {
                let text = text.strip_suffix('\n').unwrap_or(&text);
                let text = literal(text);
                self.block(match lang {
                    Some(lang) => format!("#raw(block: true, lang: {}, {text})", literal(&lang)),
                    None => format!("#raw(block: true, {text})"),
                });
            }
            Frame::Html { text, line } => {
                // Silent only for comments alone: what shares a comment's block is lost with it.
                let rest = without_comments(&text);
                if let Some(first) = rest.lines().map(str::trim).find(|l| !l.is_empty()) {
                    self.converted
                        .warnings
                        .push(format!("line {line}: raw HTML left out: {first}"));
                }
            }
            Frame::List { start, items } => {
                let items = items.join(", ");
                self.block(match start {
                    None => format!("#list({items})"),
                    Some(1) => format!("#enum({items})"),
                    Some(start) => format!("#enum(start: {start}, {items})"),
                });
            }
            Frame::Item => {
                if let Some(Open {
                    frame: Frame::List { items, .. },
                    ..
                }) = self.stack.last_mut()
                {
                    items.push(format!("[{out}]"));
                }
            }
            Frame::Table {
                align,
                header,
                cells,
            } => {
                let mut parts = vec![format!("columns: {}", align.len().max(1))];
                if align.iter().any(|a| *a != Alignment::None) {
                    let words: Vec<&str> = align
                        .iter()
                        .map(|a| match a {
                            Alignment::None => "auto",
                            Alignment::Left => "left",
                            Alignment::Center => "center",
                            Alignment::Right => "right",
                        })
                        .collect();
                    // A one-element array needs its trailing comma.
                    let comma = if words.len() == 1 { "," } else { "" };
                    parts.push(format!("align: ({}{comma})", words.join(", ")));
                }
                if !header.is_empty() {
                    parts.push(format!("table.header({})", header.join(", ")));
                }
                parts.extend(cells);
                self.block(format!("#table({})", parts.join(", ")));
            }
            Frame::Head | Frame::Row => {}
            Frame::Cell => {
                let cell = format!("[{out}]");
                let in_head = matches!(
                    self.stack.last(),
                    Some(Open {
                        frame: Frame::Head,
                        ..
                    })
                );
                if let Some(Open {
                    frame: Frame::Table { header, cells, .. },
                    ..
                }) = self
                    .stack
                    .iter_mut()
                    .rev()
                    .find(|open| matches!(open.frame, Frame::Table { .. }))
                {
                    if in_head {
                        header.push(cell);
                    } else {
                        cells.push(cell);
                    }
                }
            }
            Frame::Emphasis => self.inline(&format!("#emph[{out}]"), &plain),
            Frame::Strong => self.inline(&format!("#strong[{out}]"), &plain),
            Frame::Strike => self.inline(&format!("#strike[{out}]"), &plain),
            Frame::Link { url: Some(url) } => {
                self.inline(&format!("#link({})[{out}]", literal(&url)), &plain);
            }
            Frame::Link { url: None } | Frame::Plain => self.inline(&out, &plain),
            Frame::Picture { path: None } => self.inline(&out, &plain),
            Frame::Picture { path: Some(path) } => {
                if !self.converted.pictures.contains(&path) {
                    self.converted.pictures.push(path.clone());
                }
                let image = literal(&path);
                let figure = if out.is_empty() {
                    format!("#figure(image({image}, width: 100%))")
                } else {
                    format!("#figure(image({image}, width: 100%), caption: [{out}])")
                };
                // The paragraph it sits in, under a link or an emphasis too, keeps its figure for
                // its end; its line keeps the picture as an inline box meanwhile.
                let paragraph =
                    self.stack
                        .iter_mut()
                        .rev()
                        .find_map(|open| match &mut open.frame {
                            Frame::Paragraph { figures, .. } => Some(figures),
                            _ => None,
                        });
                match paragraph {
                    Some(figures) => {
                        figures.push(figure);
                        if let Some(top) = self.stack.last_mut() {
                            top.out
                                .push_str(&format!("#box(image({image}, height: 1em))"));
                        }
                    }
                    None => self.block(figure),
                }
            }
        }
    }
}

/// Whether a link's address has a scheme (`https:`, `mailto:`): a relative one names a file of
/// the repository or an anchor, which a PDF cannot follow, so its text stands alone.
fn has_scheme(url: &str) -> bool {
    url.split_once(':').is_some_and(|(scheme, _)| {
        scheme.len() > 1
            && scheme.starts_with(|c: char| c.is_ascii_alphabetic())
            && scheme
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || matches!(c, '+' | '-' | '.'))
    })
}

/// A picture's path from `docs`, with `/`, as GitHub reads it: a query or a fragment (such as
/// `#gh-dark-mode-only`) dropped and `%20` decoded; refused when it has a scheme, is absolute or
/// leaves `docs`, since Typst reads nothing outside its root.
fn picture_path(url: &str) -> Result<String, &'static str> {
    let url = url.trim();
    if has_scheme(url) {
        return Err("only a file in docs is read, never an address or an absolute path");
    }
    let bare = url.split(['?', '#']).next().unwrap_or_default();
    let path = percent_decoded(bare).replace('\\', "/");
    if path.trim().is_empty() {
        return Err("it names no file");
    }
    if path.starts_with('/') || path.get(1..2) == Some(":") {
        return Err("only a file in docs is read, never an address or an absolute path");
    }
    if path.split('/').any(|part| part == "..") {
        return Err("only a file in docs is read, never one outside it");
    }
    let parts: Vec<&str> = path
        .split('/')
        .filter(|part| !part.is_empty() && *part != ".")
        .collect();
    Ok(parts.join("/"))
}

/// `%XX` escapes decoded; anything else, a stray `%` included, kept as it is.
fn percent_decoded(text: &str) -> String {
    let bytes = text.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        let hex = |at: usize| bytes.get(at).and_then(|b| (*b as char).to_digit(16));
        match (bytes[i], hex(i + 1), hex(i + 2)) {
            (b'%', Some(high), Some(low)) => {
                out.push((high * 16 + low) as u8);
                i += 3;
            }
            (byte, _, _) => {
                out.push(byte);
                i += 1;
            }
        }
    }
    String::from_utf8_lossy(&out).into_owned()
}

/// Raw HTML without its comments, an unclosed one running to the end.
fn without_comments(html: &str) -> String {
    let mut out = String::new();
    let mut rest = html;
    while let Some(start) = rest.find("<!--") {
        out.push_str(&rest[..start]);
        rest = match rest[start + 4..].find("-->") {
            Some(end) => &rest[start + 4 + end + 3..],
            None => "",
        };
    }
    out.push_str(rest);
    out
}

/// A picture's file: its path from `docs` joined part by part, so that a `/` never stays inside
/// one name (a verbatim `\\?\` folder would read it as a character).
fn in_docs(docs: &Path, picture: &str) -> PathBuf {
    picture
        .split('/')
        .fold(docs.to_path_buf(), |path, part| path.join(part))
}

/// What the template needs besides the converted Markdown.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Meta<'a> {
    /// The project's name: the document's author, and its title without a `#` heading.
    pub project: &'a str,
    pub which: Which,
    /// The manual's date, `YYYY-MM-DD`: its last commit's, else the file's.
    pub date: &'a str,
}

/// The whole Typst document: the template (EP-56's trial), the title page with the title, the
/// intro and the date, the contents, then the chapters.
pub fn document(converted: &Converted, meta: &Meta) -> String {
    let title = converted
        .title
        .clone()
        .filter(|title| !title.is_empty())
        .unwrap_or_else(|| meta.which.fallback_title(meta.project));
    let mut out = String::new();
    out.push_str(&format!(
        "// Built by crw manual from {} with Typst {TYPST}; generated, never edited.\n",
        meta.which.source()
    ));
    out.push_str(&format!("#let title = {}\n", literal(&title)));
    out.push_str(&format!("#let built = {}\n", literal(meta.date)));
    out.push_str(&format!(
        "#let source = {}\n",
        literal(&meta.which.source())
    ));
    // A manual's chapter starts a page; the changes' sections, one per brief, follow each other.
    out.push_str(&format!(
        "#let chapters = {}\n",
        meta.which == Which::Manual
    ));
    out.push_str(&format!(
        "#set document(title: title, author: {})\n",
        literal(meta.project)
    ));
    out.push_str(TEMPLATE);
    out.push_str(&converted.intro);
    out.push_str(TITLE_END);
    out.push_str(&converted.body);
    out.push('\n');
    out
}

/// The template between the document's variables and the title page's text: the page, the
/// fonts, the headings (a page break before each of a manual's chapters), the tables, then the
/// title page's head, its title unjustified.
const TEMPLATE: &str = r#"#set page(
  paper: "us-letter",
  margin: (x: 2.2cm, y: 2.4cm),
  header: context {
    if counter(page).get().first() > 1 {
      set text(size: 9pt, fill: luma(90))
      [#title#h(1fr)#built]
    }
  },
  footer: context {
    if counter(page).get().first() > 1 {
      set text(size: 9pt)
      align(center, counter(page).display("1 / 1", both: true))
    }
  },
)
#set text(font: "Libertinus Serif", size: 11pt, lang: "en")
#set par(justify: true)
#show raw: set text(font: "DejaVu Sans Mono", size: 9pt)
#show raw.where(block: true): block.with(fill: luma(244), inset: 8pt, radius: 3pt, width: 100%)
#set heading(numbering: "1.1")
#show heading.where(level: 1, outlined: true): it => { if chapters { pagebreak(weak: true) }; it }
#show link: underline
#set table(stroke: 0.5pt + luma(160), inset: 5pt)
#show table.cell.where(y: 0): strong

#page(header: none, footer: none)[
  #v(22%)
  #align(center)[
    #set par(justify: false)
    #text(size: 24pt, weight: "bold", title)
  ]
  #v(18pt)
  #align(center, block(width: 85%)[
    #set align(left)

"#;

/// The title page's foot and the contents.
const TITLE_END: &str = r#"

  ])
  #v(18pt)
  #align(center, text(size: 10pt, fill: luma(90), source + ", " + built))
]

#outline(depth: 2)

"#;

/// What runs the build's children: the real `typst` and `git`, or a test's fake.
pub trait Runner {
    /// What `typst --version` prints; `Err` when Typst is not found or does not answer.
    fn version(&self) -> Result<String, String>;
    /// Runs `typst` with `args`, `input` on its standard input: how it ended.
    fn compile(&self, args: &[OsString], input: &str) -> Result<Exit, String>;
    /// Whether the working file `path` (from the project's folder, with `/`) differs from its
    /// last commit, or git does not track it: then the file's own time dates it.
    fn changed(&self, folder: &Path, path: &str) -> bool;
    /// The time of the last commit that touched `path`, in seconds since 1970; `None` when git
    /// has none.
    fn committed(&self, folder: &Path, path: &str) -> Option<i64>;
}

/// Where Typst is: on PATH, else in winget's `Links` folder, where its portable package links it.
pub fn program() -> Option<PathBuf> {
    crate::doctor::on_path("typst.exe").or_else(|| {
        let local = std::env::var_os("LOCALAPPDATA").filter(|v| !v.is_empty())?;
        let link = PathBuf::from(local).join(r"Microsoft\WinGet\Links\typst.exe");
        link.is_file().then_some(link)
    })
}

/// The real children, each in its turn (D-29): Typst where [`program`] finds it, and git.
pub struct Real {
    program: Option<PathBuf>,
}

impl Real {
    pub fn locate() -> Self {
        Self { program: program() }
    }

    fn program(&self) -> Result<&Path, String> {
        self.program.as_deref().ok_or_else(|| {
            format!(
                "Typst is not found on PATH or in winget's Links folder: install it with {INSTALL}"
            )
        })
    }
}

impl Runner for Real {
    fn version(&self) -> Result<String, String> {
        let program = self.program()?;
        let _turn = process::turn();
        process::output_of(program, ["--version"], process::TIMEOUT)
            .map_err(|why| format!("Typst at {} {why}", program.display()))
    }

    fn compile(&self, args: &[OsString], input: &str) -> Result<Exit, String> {
        let program = self.program()?;
        let _turn = process::turn();
        process::exit_with_input(program, args, input, &TYPST_ENV, COMPILE_TIMEOUT)
    }

    fn changed(&self, folder: &Path, path: &str) -> bool {
        let _turn = process::turn();
        // Reading leaves the index alone (D-11), as the badges' status does.
        crate::git::run(
            folder,
            &["--no-optional-locks", "status", "--porcelain", "--", path],
        )
        .is_ok_and(|said| !said.trim().is_empty())
    }

    fn committed(&self, folder: &Path, path: &str) -> Option<i64> {
        let _turn = process::turn();
        crate::git::run(folder, &["log", "-1", "--format=%ct", "--", path])
            .ok()?
            .trim()
            .parse()
            .ok()
    }
}

/// One build asked for.
#[derive(Debug, Clone)]
pub struct Job<'a> {
    /// The project's name, which the PDF's name and the document carry.
    pub project: &'a str,
    /// The project's folder.
    pub folder: &'a Path,
    pub which: Which,
    /// The manuals folder.
    pub out: &'a Path,
    /// Build even when the PDF is newer than everything it is made of.
    pub force: bool,
    /// When the template last changed: the time of the exe that builds, so that a new build of
    /// the tool builds its PDFs again once; `None` leaves it out.
    pub template: Option<SystemTime>,
}

/// What a build did.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Built {
    /// The PDF was built: where, its size in bytes, the date it carries, and the converter's
    /// warnings followed by Typst's.
    Wrote {
        pdf: PathBuf,
        bytes: u64,
        date: String,
        warnings: Vec<String>,
    },
    /// The PDF is newer than its source, its pictures and the template: nothing was built.
    Fresh { pdf: PathBuf },
}

/// Builds a project's manual, or its changes, into the manuals folder: refused when the project
/// owes no manual (no `docs/manual.md`, D-12), when a picture it names is missing, or when Typst
/// is missing or not the pinned version; fresh, with no child at all, when the PDF is newer than
/// everything it is made of, unless forced. The PDF is written through a temporary file beside
/// it and renamed into place, so a failed build leaves the last good PDF as it was.
pub fn build(job: &Job, runner: &dyn Runner) -> Result<Built, String> {
    let docs = job.folder.join(DOCS);
    if !docs.join(Which::Manual.file()).is_file() {
        return Err(format!(
            "{} owes no manual: it has no {} (D-12)",
            job.project,
            Which::Manual.source()
        ));
    }
    let source = docs.join(job.which.file());
    if !source.is_file() {
        return Err(format!(
            "{} has {} but no {}: the session that settled its manual creates both",
            job.project,
            Which::Manual.source(),
            job.which.source()
        ));
    }
    let text = fs::read_to_string(&source)
        .map_err(|e| format!("could not read {}: {e}", source.display()))?;
    let converted = convert(&text);
    let missing: Vec<&str> = converted
        .pictures
        .iter()
        .filter(|picture| !in_docs(&docs, picture).is_file())
        .map(String::as_str)
        .collect();
    if !missing.is_empty() {
        return Err(format!(
            "{} names pictures that are not in {DOCS}: {}",
            job.which.source(),
            missing.join(", ")
        ));
    }
    let pdf = job.out.join(job.which.pdf(job.project));
    if !job.force && fresh(&pdf, &source, &docs, &converted.pictures, job.template) {
        return Ok(Built::Fresh { pdf });
    }
    let said = runner.version()?;
    match Version::find_in(&said) {
        Some(version) if version == TYPST => {}
        Some(version) => {
            return Err(format!(
                "Typst {version} is installed, but crw manual builds with {TYPST} alone, the \
                 version its template is checked under (D-12): {INSTALL}, then {PIN}"
            ));
        }
        None => {
            return Err(format!(
                "Typst printed no version ({}): {INSTALL}",
                said.trim()
            ));
        }
    }
    // The last commit's time while the file is as committed; an edit not yet committed (a
    // re-plan builds before it commits) or an untracked file is dated by the file itself.
    let path = job.which.source();
    let committed = if runner.changed(job.folder, &path) {
        None
    } else {
        runner.committed(job.folder, &path)
    };
    let stamp = committed
        .or_else(|| modified(&source).map(seconds))
        .unwrap_or(0);
    let date = date_of(stamp);
    let typst = document(
        &converted,
        &Meta {
            project: job.project,
            which: job.which,
            date: &date,
        },
    );
    fs::create_dir_all(job.out)
        .map_err(|e| format!("could not create {}: {e}", job.out.display()))?;
    let stem = job.which.pdf(job.project);
    let stem = stem.strip_suffix(".pdf").unwrap_or(&stem);
    let temp = job.out.join(format!("{stem}.building.pdf"));
    let args: Vec<OsString> = vec![
        "compile".into(),
        "--root".into(),
        docs.clone().into_os_string(),
        "--ignore-system-fonts".into(),
        "--creation-timestamp".into(),
        stamp.to_string().into(),
        "-".into(),
        temp.clone().into_os_string(),
    ];
    let failed = |words: String| {
        let _ = fs::remove_file(&temp);
        Err(words)
    };
    let exit = match runner.compile(&args, &typst) {
        Ok(exit) => exit,
        Err(why) => return failed(format!("typst compile {why}")),
    };
    if exit.code != Some(0) {
        let said: Vec<&str> = exit
            .stderr
            .lines()
            .map(str::trim_end)
            .filter(|line| !line.trim().is_empty())
            .take(12)
            .collect();
        let code = exit
            .code
            .map_or_else(|| "no code".to_owned(), |code| code.to_string());
        return failed(format!(
            "typst compile failed (exit {code}): {}",
            said.join(" | ")
        ));
    }
    if !temp.is_file() {
        return failed("typst compile ended well but wrote no PDF".to_owned());
    }
    if let Err(e) = fs::rename(&temp, &pdf) {
        return failed(format!(
            "could not put the PDF in place at {}: {e}; a viewer may hold it open: close it and \
             build again",
            pdf.display()
        ));
    }
    let bytes = fs::metadata(&pdf).map(|m| m.len()).unwrap_or(0);
    // Typst's own warnings follow the converter's: a construct it leaves out says so.
    let mut warnings = converted.warnings;
    warnings.extend(
        exit.stderr
            .lines()
            .map(str::trim)
            .filter(|line| line.starts_with("warning:"))
            .map(|line| format!("typst {line}")),
    );
    Ok(Built::Wrote {
        pdf,
        bytes,
        date,
        warnings,
    })
}

/// Whether the PDF is newer than its source, every picture and the template.
fn fresh(
    pdf: &Path,
    source: &Path,
    docs: &Path,
    pictures: &[String],
    template: Option<SystemTime>,
) -> bool {
    let Some(built) = modified(pdf) else {
        return false;
    };
    let mut inputs = vec![modified(source)];
    inputs.extend(
        pictures
            .iter()
            .map(|picture| modified(&in_docs(docs, picture))),
    );
    inputs.push(template.or(Some(UNIX_EPOCH)));
    inputs
        .into_iter()
        .all(|input| input.is_some_and(|time| time <= built))
}

fn modified(path: &Path) -> Option<SystemTime> {
    fs::metadata(path).ok()?.modified().ok()
}

fn seconds(time: SystemTime) -> i64 {
    time.duration_since(UNIX_EPOCH)
        .map_or(0, |since| since.as_secs() as i64)
}

/// A timestamp's local date, `YYYY-MM-DD`.
pub fn date_of(stamp: i64) -> String {
    DateTime::from_timestamp(stamp, 0)
        .map(|time| time.with_timezone(&Local).format("%Y-%m-%d").to_string())
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::cell::RefCell;

    use crate::files::tests::Scratch;

    /// Every string literal in `typst` is followed by `#`, `]`, `)`, `,`, a space or a line's
    /// end, never by a character that would continue it as code (`.` reads as a field access).
    fn followers_hold(typst: &str) {
        let bytes: Vec<char> = typst.chars().collect();
        let mut i = 0;
        let mut in_literal = false;
        while i < bytes.len() {
            let c = bytes[i];
            if in_literal {
                if c == '\\' {
                    i += 2;
                    continue;
                }
                if c == '"' {
                    in_literal = false;
                    let next = bytes.get(i + 1).copied();
                    assert!(
                        matches!(next, None | Some('#' | ']' | ')' | ',' | '\n' | ' ')),
                        "a literal followed by {next:?} at {i} in {typst}"
                    );
                }
            } else if c == '"' {
                in_literal = true;
            }
            i += 1;
        }
    }

    fn body(markdown: &str) -> String {
        let converted = convert(markdown);
        followers_hold(&converted.body);
        followers_hold(&converted.intro);
        converted.body
    }

    #[test]
    fn a_literal_escapes_only_what_typst_strings_read() {
        assert_eq!(literal(r#"a "b" \c"#), r#""a \"b\" \\c""#);
        assert_eq!(
            literal("# $ * _ @ < > ` ~ = - + / [ ]"),
            "\"# $ * _ @ < > ` ~ = - + / [ ]\""
        );
        assert_eq!(literal("one\r\ntwo\tthree"), r#""one\ntwo\tthree""#);
        assert_eq!(literal("bell\u{7}"), r#""bell\u{7}""#);
        assert_eq!(literal("“curly” — dash"), "\"“curly” — dash\"");
    }

    #[test]
    fn the_title_and_the_intro_go_to_the_title_page() {
        let converted = convert(
            "# Example — manual\n\nWhat it is, as the phase P6 leaves it.\n\n## What it does\n\nText.\n",
        );
        assert_eq!(converted.title.as_deref(), Some("Example — manual"));
        assert_eq!(
            converted.intro,
            "#par[#\"What it is, as the phase P6 leaves it.\"]"
        );
        assert_eq!(
            converted.body,
            "#heading(level: 1)[#\"What it does\"]\n\n#par[#\"Text.\"]"
        );
        assert!(converted.warnings.is_empty());
    }

    #[test]
    fn headings_step_down_one_level_and_a_second_title_is_a_chapter() {
        let converted = convert("# T\n\n## One\n\n### Two\n\n#### Three\n\n# Again\n");
        assert_eq!(
            converted.body,
            "#heading(level: 1)[#\"One\"]\n\n#heading(level: 2)[#\"Two\"]\n\n\
             #heading(level: 3)[#\"Three\"]\n\n#heading(level: 1)[#\"Again\"]"
        );
        assert_eq!(
            converted.warnings,
            ["line 9: a second # heading, read as a chapter: the manual's title is its first"]
        );
    }

    /// The text's own characters, joined across the literals pulldown-cmark splits it into.
    fn text_of(typst: &str) -> String {
        let mut out = String::new();
        let mut rest = typst;
        while let Some(start) = rest.find("#\"") {
            let after = &rest[start + 2..];
            let mut end = 0;
            let mut chars = after.char_indices();
            while let Some((i, c)) = chars.next() {
                match c {
                    '\\' => {
                        if let Some((_, escaped)) = chars.next() {
                            out.push(match escaped {
                                'n' => '\n',
                                't' => '\t',
                                other => other,
                            });
                        }
                    }
                    '"' => {
                        end = i;
                        break;
                    }
                    c => out.push(c),
                }
            }
            rest = &after[end + 1..];
        }
        out
    }

    #[test]
    fn typst_syntax_in_the_text_stays_text() {
        let markdown =
            "## H\n\nA # $x$ @who a < b > c \\\\ ~ = - + / [x] {y} \"q\" 'q' -- 1. * _ ` .\n";
        let text = body(markdown);
        assert!(text.starts_with("#heading(level: 1)[#\"H\"]\n\n#par[#\"A # $x$ @who "));
        assert_eq!(
            text_of(&text["#heading(level: 1)[#\"H\"]".len()..]),
            "A # $x$ @who a < b > c \\ ~ = - + / [x] {y} \"q\" 'q' -- 1. * _ ` ."
        );
        assert!(convert(markdown).warnings.is_empty());
    }

    #[test]
    fn inline_marks_links_and_breaks() {
        let text = body(
            "## H\n\n**Strong** and ~~gone~~, `crw doctor`, [the site](https://example.com), \
             [a file](other.md),\nnext line  \nhard.\n",
        );
        assert_eq!(
            text,
            "#heading(level: 1)[#\"H\"]\n\n#par[#strong[#\"Strong\"]#\" and \"#strike[#\"gone\"]\
             #\", \"#raw(\"crw doctor\")#\", \"#link(\"https://example.com\")[#\"the site\"]\
             #\", \"#\"a file\"#\",\"#\" \"#\"next line\"#linebreak()#\"hard.\"]"
        );
    }

    #[test]
    fn lists_nest_and_keep_their_start() {
        // A new delimiter starts a new list (CommonMark), so the last is a list of its own.
        let text = body("## H\n\n- one\n  - inner\n- two\n\n3. third\n4. fourth\n\n1) first\n");
        assert_eq!(
            text,
            "#heading(level: 1)[#\"H\"]\n\n#list([#\"one\"#list([#\"inner\"])], [#\"two\"])\n\n\
             #enum(start: 3, [#\"third\"], [#\"fourth\"])\n\n#enum([#\"first\"])"
        );
    }

    #[test]
    fn a_loose_list_keeps_its_paragraphs() {
        let text = body("## H\n\n- one\n\n  more\n\n- two\n");
        assert_eq!(
            text,
            "#heading(level: 1)[#\"H\"]\n\n#list([#par[#\"one\"]#par[#\"more\"]], [#par[#\"two\"]])"
        );
    }

    #[test]
    fn a_pipe_table_keeps_its_header_and_alignment() {
        let text = body(
            "## H\n\n| Size | Minutes | Use |\n|:--|--:|:-:|\n| S | 30 | `known` |\n| M | 45 | an *aim* |\n",
        );
        assert_eq!(
            text,
            "#heading(level: 1)[#\"H\"]\n\n#table(columns: 3, align: (left, right, center), \
             table.header([#\"Size\"], [#\"Minutes\"], [#\"Use\"]), [#\"S\"], [#\"30\"], \
             [#raw(\"known\")], [#\"M\"], [#\"45\"], [#\"an \"#emph[#\"aim\"]])"
        );
        // An escaped pipe is the cell's text.
        let piped = convert("| A |\n|---|\n| a \\| b |\n").intro;
        assert_eq!(text_of(&piped), "Aa | b");
        assert_eq!(body("| A |\n|---|\n| b |\n"), "");
        assert_eq!(
            convert("| A |\n|---|\n| b |\n").intro,
            "#table(columns: 1, table.header([#\"A\"]), [#\"b\"])"
        );
        assert_eq!(
            convert("| A |\n|:-:|\n| b |\n").intro,
            "#table(columns: 1, align: (center,), table.header([#\"A\"]), [#\"b\"])"
        );
    }

    #[test]
    fn a_picture_alone_is_a_figure_and_amid_text_stays_in_the_line() {
        let converted = convert(
            "## H\n\n![The list, *light*](dashboard-light.png)\n\nAn icon ![i](pics\\icon.png) here, \
             ![again](dashboard-light.png).\n",
        );
        followers_hold(&converted.body);
        assert_eq!(
            converted.body,
            "#heading(level: 1)[#\"H\"]\n\n#figure(image(\"dashboard-light.png\", width: 100%), \
             caption: [#\"The list, \"#emph[#\"light\"]])\n\n#par[#\"An icon \"\
             #box(image(\"pics/icon.png\", height: 1em))#\" here, \"\
             #box(image(\"dashboard-light.png\", height: 1em))#\".\"]"
        );
        assert_eq!(converted.pictures, ["dashboard-light.png", "pics/icon.png"]);
        let bare = convert("![](./shot.png)\n");
        assert_eq!(bare.intro, "#figure(image(\"shot.png\", width: 100%))");
    }

    /// A picture under a link or an emphasis, alone in its paragraph, is a figure too, never a
    /// figure inside a paragraph (which Typst leaves out); two pictures alone are two figures;
    /// amid text a linked picture stays in the line.
    #[test]
    fn a_picture_is_judged_by_its_paragraph_not_its_parent() {
        let linked = convert("[![Shot](shot.png)](shot.png)\n");
        assert_eq!(
            linked.intro,
            "#figure(image(\"shot.png\", width: 100%), caption: [#\"Shot\"])"
        );
        let emphasised = convert("*![a](a.png)*\n");
        assert_eq!(
            emphasised.intro,
            "#figure(image(\"a.png\", width: 100%), caption: [#\"a\"])"
        );
        let two = convert("![light](light.png)\n![dark](dark.png)\n");
        assert_eq!(
            two.intro,
            "#figure(image(\"light.png\", width: 100%), caption: [#\"light\"])\n\n\
             #figure(image(\"dark.png\", width: 100%), caption: [#\"dark\"])"
        );
        let amid = convert("See [![i](i.png)](https://example.com) here.\n");
        followers_hold(&amid.intro);
        assert_eq!(
            amid.intro,
            "#par[#\"See \"#link(\"https://example.com\")[#box(image(\"i.png\", height: 1em))]\
             #\" here.\"]"
        );
        assert!(!amid.intro.contains("#figure"));
    }

    /// Paths as GitHub reads them: a fragment or a query dropped, `%20` decoded, `./` parts gone.
    #[test]
    fn picture_paths_are_read_as_github_reads_them() {
        assert_eq!(picture_path("my%20shot.png"), Ok("my shot.png".to_owned()));
        assert_eq!(
            picture_path("shot.png#gh-dark-mode-only"),
            Ok("shot.png".to_owned())
        );
        assert_eq!(picture_path("shot.png?raw=true"), Ok("shot.png".to_owned()));
        assert_eq!(picture_path("./pics/./a.png"), Ok("pics/a.png".to_owned()));
        assert_eq!(picture_path("pics\\a.png"), Ok("pics/a.png".to_owned()));
        assert_eq!(picture_path("100%.png"), Ok("100%.png".to_owned()));
        assert!(picture_path("%2e%2e/secret.png").is_err());
        assert!(picture_path("#only-a-fragment").is_err());
        assert!(picture_path("https://example.com/a.png").is_err());
        assert!(picture_path("/abs.png").is_err());
        assert!(picture_path("C:\\abs.png").is_err());
        let docs = Path::new(r"\\?\C:\Data\alpha\docs");
        assert_eq!(in_docs(docs, "pics/a.png"), docs.join("pics").join("a.png"));
    }

    /// A heading inside a quote or a list starts no page and stays out of the contents; a `#`
    /// that is not the title is worded by why.
    #[test]
    fn a_nested_heading_starts_no_page_and_a_stray_title_says_why() {
        let quoted = convert("> ## Tip\n>\n> Text.\n\n## One\n");
        assert_eq!(
            quoted.intro,
            "#quote(block: true)[#heading(level: 1, outlined: false, numbering: none)[#\"Tip\"]\
             #par[#\"Text.\"]]"
        );
        assert_eq!(quoted.body, "#heading(level: 1)[#\"One\"]");
        let late = convert("## One\n\n# Late\n");
        assert_eq!(late.title, None);
        assert!(
            late.warnings[0].contains("after the first chapter"),
            "{:?}",
            late.warnings
        );
        let inside = convert("> # Quoted\n\n## One\n");
        assert_eq!(inside.title, None);
        assert!(
            inside.warnings[0].contains("inside a quote or a list"),
            "{:?}",
            inside.warnings
        );
        let second = convert("# T\n\n# Again\n");
        assert!(
            second.warnings[0].contains("a second # heading"),
            "{:?}",
            second.warnings
        );
    }

    #[test]
    fn a_picture_outside_docs_is_left_out_with_a_warning() {
        let converted = convert(
            "## H\n\n![web](https://example.com/a.png)\n\n![up](../secret.png)\n\n![abs](C:/x.png)\n",
        );
        assert!(converted.pictures.is_empty());
        assert_eq!(
            converted.body,
            "#heading(level: 1)[#\"H\"]\n\n#par[#\"web\"]\n\n#par[#\"up\"]\n\n#par[#\"abs\"]"
        );
        assert_eq!(converted.warnings.len(), 3);
        assert!(converted.warnings[0].starts_with("line 3: the picture https://example.com/a.png"));
        assert!(converted.warnings[1].starts_with("line 5: the picture ../secret.png"));
        assert!(converted.warnings[2].contains("absolute path"));
    }

    #[test]
    fn quotes_code_blocks_and_rules() {
        let text = body(
            "## H\n\n> Quoted *words*.\n\n```powershell\ncrw manual \"a\" \\x\n```\n\n    indented\n\n---\n",
        );
        assert_eq!(
            text,
            "#heading(level: 1)[#\"H\"]\n\n#quote(block: true)[#par[#\"Quoted \"#emph[#\"words\"]#\".\"]]\n\n\
             #raw(block: true, lang: \"powershell\", \"crw manual \\\"a\\\" \\\\x\")\n\n\
             #raw(block: true, \"indented\")\n\n#line(length: 100%)"
        );
    }

    #[test]
    fn raw_html_is_left_out_with_its_line_and_a_comment_silently() {
        let converted = convert(
            "## H\n\n<!-- a note for the author -->\n\n<div align=\"center\">\nx\n</div>\n\nA <b>bold</b> word.\n",
        );
        assert_eq!(
            converted.body,
            "#heading(level: 1)[#\"H\"]\n\n#par[#\"A \"#\"bold\"#\" word.\"]"
        );
        assert_eq!(
            converted.warnings,
            [
                "line 5: raw HTML left out: <div align=\"center\">",
                "line 9: raw HTML left out: <b>",
                "line 9: raw HTML left out: </b>",
            ]
        );
        // What shares a comment's block is lost with it, and said.
        let shared = convert("<!-- note --> ![a](a.png)\n");
        assert_eq!(shared.warnings, ["line 1: raw HTML left out: ![a](a.png)"]);
        assert!(
            convert("<!-- one -->\n<!-- two\nlines -->\n")
                .warnings
                .is_empty()
        );
    }

    #[test]
    fn a_manual_without_a_title_takes_the_projects_name() {
        let converted = convert("Just text.\n");
        assert_eq!(converted.title, None);
        let typst = document(
            &converted,
            &Meta {
                project: "example",
                which: Which::Changes,
                date: "2026-11-25",
            },
        );
        assert!(
            typst.contains("#let title = \"example — changes since the last phase's close\"\n")
        );
        assert!(typst.contains("#let source = \"docs/manual-changes.md\"\n"));
        assert!(
            typst.contains("#let chapters = false\n"),
            "the changes flow on"
        );
        assert!(typst.contains("#set document(title: title, author: \"example\")\n"));
    }

    #[test]
    fn the_document_wraps_the_parts_in_the_template() {
        let converted = convert("# Example — manual\n\nIntro.\n\n## One\n\nText.\n");
        let typst = document(
            &converted,
            &Meta {
                project: "example",
                which: Which::Manual,
                date: "2026-11-25",
            },
        );
        assert!(typst.starts_with("// Built by crw manual from docs/manual.md with Typst 0.15.1"));
        assert!(typst.contains("#let title = \"Example — manual\"\n#let built = \"2026-11-25\"\n"));
        assert!(
            typst.contains("#let chapters = true\n"),
            "a chapter starts a page"
        );
        let intro = typst.find("#par[#\"Intro.\"]").unwrap();
        let outline = typst.find("#outline(depth: 2)").unwrap();
        let chapter = typst.find("#heading(level: 1)[#\"One\"]").unwrap();
        assert!(typst.find("#page(header: none, footer: none)[").unwrap() < intro);
        assert!(intro < outline && outline < chapter);
        assert!(typst.contains("paper: \"us-letter\""));
        assert!(typst.contains("counter(page).display(\"1 / 1\", both: true)"));
        assert!(!typst.contains("@preview"), "no package is imported");
        assert!(typst.ends_with("#par[#\"Text.\"]\n"));
    }

    /// A checkout with Windows line endings gives the same Typst, so the same bytes.
    #[test]
    fn line_endings_change_nothing() {
        let markdown = "# T\n\nIntro\nline.\n\n## One\n\n```\na\nb\n```\n\n- x\n- y\n\n<b>\n\n| A |\n|---|\n| b |\n";
        assert_eq!(convert(&markdown.replace('\n', "\r\n")), convert(markdown));
        assert_eq!(convert(&format!("\u{feff}{markdown}")), convert(markdown));
    }

    #[test]
    fn the_pdfs_names_and_sources() {
        assert_eq!(Which::Manual.pdf("alpha"), "alpha-manual.pdf");
        assert_eq!(Which::Changes.pdf("alpha"), "alpha-manual-changes.pdf");
        assert_eq!(Which::Manual.source(), "docs/manual.md");
        assert_eq!(Which::Changes.source(), "docs/manual-changes.md");
        assert_eq!(TYPST.to_string(), "0.15.1");
        assert!(INSTALL.ends_with(&TYPST.to_string()));
        assert!(PIN.ends_with(&TYPST.to_string()));
    }

    /// A runner that answers from a script, keeps what it was asked and writes a fake PDF where
    /// the compile names its output.
    struct Fake {
        version: Result<String, String>,
        exit: Exit,
        changed: bool,
        committed: Option<i64>,
        compiled: RefCell<Vec<(Vec<String>, String)>>,
        writes: bool,
    }

    impl Fake {
        fn new() -> Self {
            Self {
                version: Ok("typst 0.15.1 (9dfd3a08)\n".to_owned()),
                exit: Exit {
                    code: Some(0),
                    stdout: String::new(),
                    stderr: String::new(),
                },
                changed: false,
                committed: Some(1_764_072_000),
                compiled: RefCell::new(Vec::new()),
                writes: true,
            }
        }

        fn compiles(&self) -> usize {
            self.compiled.borrow().len()
        }
    }

    impl Runner for Fake {
        fn version(&self) -> Result<String, String> {
            self.version.clone()
        }

        fn compile(&self, args: &[OsString], input: &str) -> Result<Exit, String> {
            let args: Vec<String> = args
                .iter()
                .map(|arg| arg.to_string_lossy().into_owned())
                .collect();
            if self.writes {
                fs::write(args.last().unwrap(), b"%PDF-1.7 fake").unwrap();
            }
            self.compiled.borrow_mut().push((args, input.to_owned()));
            Ok(self.exit.clone())
        }

        fn changed(&self, _folder: &Path, _path: &str) -> bool {
            self.changed
        }

        fn committed(&self, _folder: &Path, _path: &str) -> Option<i64> {
            self.committed
        }
    }

    struct Project {
        _scratch: Scratch,
        folder: PathBuf,
        out: PathBuf,
    }

    /// Which of the two files are there, with the manual's own time (EP-57a): the changes file
    /// alone builds nothing, since a project without the manual owes none.
    #[test]
    fn reads_which_of_the_two_files_are_there() {
        let alpha = project("present", None);
        assert_eq!(Present::read(&alpha.folder), Present::default());
        assert!(!Present::default().has(Which::Manual));
        let docs = alpha.folder.join(DOCS);
        fs::write(docs.join("manual-changes.md"), "# Alpha\n").unwrap();
        let changes_only = Present::read(&alpha.folder);
        assert!(changes_only.manual.is_none() && changes_only.changes);
        assert!(!changes_only.has(Which::Changes), "{changes_only:?}");
        fs::write(docs.join("manual.md"), "# Alpha\n").unwrap();
        let both = Present::read(&alpha.folder);
        let own = seconds(modified(&docs.join("manual.md")).unwrap());
        assert_eq!(both.manual, Some(own));
        assert!(both.has(Which::Manual) && both.has(Which::Changes));
        // A folder named like the file is not the file.
        let beta = project("present-folder", None);
        fs::create_dir_all(beta.folder.join(DOCS).join("manual.md")).unwrap();
        assert_eq!(Present::read(&beta.folder), Present::default());
    }

    /// The viewer's path goes with the system's separators alone (EP-57a): a `CRW_HOME` written
    /// with `/` would read as a switch to Explorer; the commas stay, inside the quotes.
    #[test]
    fn the_viewers_path_has_the_systems_separators() {
        assert_eq!(
            normalised(Path::new(r"C:/scratch/crw\manuals\a,b-manual.pdf")),
            PathBuf::from(r"C:\scratch\crw\manuals\a,b-manual.pdf")
        );
        assert_eq!(
            normalised(Path::new(r"C:\Local\manuals\alpha-manual.pdf")),
            PathBuf::from(r"C:\Local\manuals\alpha-manual.pdf")
        );
    }

    fn project(name: &str, manual: Option<&str>) -> Project {
        let scratch = Scratch::new(&format!("manual-{name}"));
        let folder = scratch.0.join("alpha");
        let docs = folder.join(DOCS);
        fs::create_dir_all(&docs).unwrap();
        if let Some(manual) = manual {
            fs::write(docs.join("manual.md"), manual).unwrap();
            fs::write(
                docs.join("manual-changes.md"),
                "# Alpha — changes since the last phase's close\n\nSince EP-1.\n",
            )
            .unwrap();
        }
        let out = scratch.0.join("home").join(MANUALS);
        Project {
            folder,
            out,
            _scratch: scratch,
        }
    }

    fn job<'a>(project: &'a Project, which: Which, force: bool) -> Job<'a> {
        Job {
            project: "alpha",
            folder: &project.folder,
            which,
            out: &project.out,
            force,
            template: None,
        }
    }

    fn age(path: &Path, seconds_ago: u64) {
        let file = fs::File::options().write(true).open(path).unwrap();
        file.set_modified(SystemTime::now() - Duration::from_secs(seconds_ago))
            .unwrap();
    }

    const MANUAL: &str = "# Alpha — manual\n\nWhat it is.\n\n## One\n\n![Shot](shot.png)\n";

    #[test]
    fn builds_through_stdin_under_the_docs_root_and_renames_into_place() {
        let project = project("stdin", Some(MANUAL));
        fs::write(project.folder.join(DOCS).join("shot.png"), b"png").unwrap();
        let fake = Fake::new();
        let built = build(&job(&project, Which::Manual, false), &fake).unwrap();
        let pdf = project.out.join("alpha-manual.pdf");
        assert_eq!(
            built,
            Built::Wrote {
                pdf: pdf.clone(),
                bytes: 13,
                date: date_of(1_764_072_000),
                warnings: Vec::new(),
            }
        );
        assert_eq!(fs::read(&pdf).unwrap(), b"%PDF-1.7 fake");
        assert!(!project.out.join("alpha-manual.building.pdf").exists());
        let compiled = fake.compiled.borrow();
        let (args, input) = &compiled[0];
        let docs = project.folder.join(DOCS);
        assert_eq!(
            args,
            &[
                "compile".to_owned(),
                "--root".to_owned(),
                docs.display().to_string(),
                "--ignore-system-fonts".to_owned(),
                "--creation-timestamp".to_owned(),
                "1764072000".to_owned(),
                "-".to_owned(),
                project
                    .out
                    .join("alpha-manual.building.pdf")
                    .display()
                    .to_string(),
            ]
        );
        assert_eq!(
            input,
            &document(
                &convert(MANUAL),
                &Meta {
                    project: "alpha",
                    which: Which::Manual,
                    date: &date_of(1_764_072_000),
                }
            )
        );
    }

    #[test]
    fn a_fresh_pdf_is_left_alone_until_its_source_or_a_picture_changes() {
        let project = project("fresh", Some(MANUAL));
        let docs = project.folder.join(DOCS);
        fs::write(docs.join("shot.png"), b"png").unwrap();
        age(&docs.join("manual.md"), 600);
        age(&docs.join("shot.png"), 600);
        let fake = Fake::new();
        build(&job(&project, Which::Manual, false), &fake).unwrap();
        let pdf = project.out.join("alpha-manual.pdf");
        assert_eq!(
            build(&job(&project, Which::Manual, false), &fake).unwrap(),
            Built::Fresh { pdf: pdf.clone() }
        );
        assert_eq!(fake.compiles(), 1, "a fresh PDF asks no child");
        // A picture newer than the PDF builds it again.
        age(&pdf, 300);
        age(&docs.join("shot.png"), 60);
        assert!(matches!(
            build(&job(&project, Which::Manual, false), &fake).unwrap(),
            Built::Wrote { .. }
        ));
        // So does the source, and --force builds anyway.
        age(&pdf, 300);
        age(&docs.join("shot.png"), 600);
        age(&docs.join("manual.md"), 60);
        assert!(matches!(
            build(&job(&project, Which::Manual, false), &fake).unwrap(),
            Built::Wrote { .. }
        ));
        assert!(matches!(
            build(&job(&project, Which::Manual, true), &fake).unwrap(),
            Built::Wrote { .. }
        ));
        // And a template newer than the PDF: a new build of the tool.
        let mut newer = job(&project, Which::Manual, false);
        newer.template = Some(SystemTime::now() + Duration::from_secs(60));
        assert!(matches!(build(&newer, &fake).unwrap(), Built::Wrote { .. }));
        assert_eq!(fake.compiles(), 5);
    }

    #[test]
    fn the_changes_build_their_own_pdf() {
        let project = project("changes", Some(MANUAL));
        fs::write(project.folder.join(DOCS).join("shot.png"), b"png").unwrap();
        let fake = Fake::new();
        let built = build(&job(&project, Which::Changes, false), &fake).unwrap();
        assert!(
            matches!(built, Built::Wrote { ref pdf, .. } if pdf.ends_with("alpha-manual-changes.pdf"))
        );
        let compiled = fake.compiled.borrow();
        assert!(
            compiled[0]
                .1
                .contains("#let source = \"docs/manual-changes.md\"")
        );
        assert!(
            compiled[0]
                .1
                .contains("#let title = \"Alpha — changes since the last phase's close\"")
        );
    }

    #[test]
    fn a_project_without_a_manual_owes_none() {
        let project = project("none", None);
        let fake = Fake::new();
        let why = build(&job(&project, Which::Manual, false), &fake).unwrap_err();
        assert_eq!(why, "alpha owes no manual: it has no docs/manual.md (D-12)");
        fs::write(project.folder.join(DOCS).join("manual.md"), MANUAL).unwrap();
        let why = build(&job(&project, Which::Changes, false), &fake).unwrap_err();
        assert!(why.contains("but no docs/manual-changes.md"), "{why}");
        assert_eq!(fake.compiles(), 0);
    }

    #[test]
    fn a_missing_picture_is_refused_by_its_path() {
        let project = project("picture", Some(MANUAL));
        let fake = Fake::new();
        let why = build(&job(&project, Which::Manual, false), &fake).unwrap_err();
        assert_eq!(
            why,
            "docs/manual.md names pictures that are not in docs: shot.png"
        );
        assert_eq!(fake.compiles(), 0);
    }

    #[test]
    fn typst_missing_or_at_another_version_is_refused_with_the_winget_line() {
        let project = project("typst", Some("# A\n"));
        let mut fake = Fake::new();
        fake.version = Ok("typst 0.16.0 (0000000)".to_owned());
        let why = build(&job(&project, Which::Manual, false), &fake).unwrap_err();
        assert!(why.starts_with("Typst 0.16.0 is installed, but crw manual builds with 0.15.1"));
        assert!(why.contains(INSTALL) && why.contains(PIN), "{why}");
        fake.version = Err(Real { program: None }.program().unwrap_err());
        let why = build(&job(&project, Which::Manual, false), &fake).unwrap_err();
        assert!(why.starts_with("Typst is not found"), "{why}");
        assert!(why.ends_with(INSTALL), "{why}");
        assert_eq!(fake.compiles(), 0);
    }

    #[test]
    fn a_failed_compile_leaves_the_last_pdf_and_no_temporary_file() {
        let project = project("failed", Some("# A\n"));
        fs::create_dir_all(&project.out).unwrap();
        let pdf = project.out.join("alpha-manual.pdf");
        fs::write(&pdf, b"the last good one").unwrap();
        let mut fake = Fake::new();
        fake.exit = Exit {
            code: Some(1),
            stdout: String::new(),
            stderr: "error: file not found\n  ┌─ <stdin>:3:9\n\n".to_owned(),
        };
        let why = build(&job(&project, Which::Manual, true), &fake).unwrap_err();
        assert_eq!(
            why,
            "typst compile failed (exit 1): error: file not found |   ┌─ <stdin>:3:9"
        );
        assert_eq!(fs::read(&pdf).unwrap(), b"the last good one");
        assert!(!project.out.join("alpha-manual.building.pdf").exists());
        // Ended well without a PDF: said, nothing renamed.
        fake.exit.code = Some(0);
        fake.writes = false;
        let why = build(&job(&project, Which::Manual, true), &fake).unwrap_err();
        assert_eq!(why, "typst compile ended well but wrote no PDF");
        assert_eq!(fs::read(&pdf).unwrap(), b"the last good one");
    }

    #[test]
    fn the_timestamp_is_the_files_own_time_when_git_has_none() {
        let project = project("stamp", Some("# A\n"));
        let source = project.folder.join(DOCS).join("manual.md");
        let file = fs::File::options().write(true).open(&source).unwrap();
        file.set_modified(UNIX_EPOCH + Duration::from_secs(1_700_000_000))
            .unwrap();
        drop(file);
        let mut fake = Fake::new();
        fake.committed = None;
        let built = build(&job(&project, Which::Manual, false), &fake).unwrap();
        assert!(matches!(built, Built::Wrote { ref date, .. } if *date == date_of(1_700_000_000)));
        assert_eq!(fake.compiled.borrow()[0].0[5], "1700000000");
        // An edit not yet committed (a re-plan builds before it commits) is dated by the file,
        // not by the commit before it.
        fake.committed = Some(1_600_000_000);
        fake.changed = true;
        build(&job(&project, Which::Manual, true), &fake).unwrap();
        assert_eq!(fake.compiled.borrow()[1].0[5], "1700000000");
        // As committed, the commit dates it.
        fake.changed = false;
        build(&job(&project, Which::Manual, true), &fake).unwrap();
        assert_eq!(fake.compiled.borrow()[2].0[5], "1600000000");
    }

    /// Typst's own warnings on a build that ends well follow the converter's.
    #[test]
    fn typsts_warnings_follow_the_converters() {
        let project = project("warned", Some("# A\n\n<b>x</b>\n"));
        let mut fake = Fake::new();
        fake.exit.stderr =
            "warning: a block may not occur inside of a paragraph and was ignored\n  \
                            ┌─ <stdin>:40:7\n\n"
                .to_owned();
        let built = build(&job(&project, Which::Manual, false), &fake).unwrap();
        let Built::Wrote { warnings, .. } = built else {
            panic!("built");
        };
        assert_eq!(
            warnings,
            [
                "line 3: raw HTML left out: <b>",
                "line 3: raw HTML left out: </b>",
                "typst warning: a block may not occur inside of a paragraph and was ignored",
            ]
        );
    }
}
