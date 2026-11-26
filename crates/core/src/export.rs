//! The exports (D-8, D-24; EP-27). The site's `projects.json` for the public projects, written
//! into the site repository's `assets/data` on the owner's yes, which the site's page reads (the
//! tool never edits the page); a data repository's card taken from the file it writes itself in
//! the tool's own `cards` folder, nothing inside the repository read (EP-58b); the daily JSON
//! export of the registry and the ledger into this repository's git-ignored
//! `private/export/<date>/`, which the owner's Documents backup archives; a project's sessions and
//! daily counts as CSV on demand. After each daily write, [`prune`] keeps the last [`KEEP_DAYS`]
//! days' folders and each month's first, removing the rest one folder at a time (EP-41).

use std::collections::BTreeSet;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::time::Duration;

use chrono::{DateTime, Datelike, NaiveDate, SecondsFormat, TimeDelta, Utc};
use serde::{Deserialize, Serialize};

use crate::badges::hosting::{Class, Hosting, Listed, Visibility};
use crate::badges::stack::Chip;
use crate::files;
use crate::ledger::{Ledger, SessionRow, SnapshotRow};
use crate::paths;
use crate::roadmap::table::Roadmap;

/// Where the site export goes in the site's repository.
pub const SITE_DATA: &str = "assets/data";
pub const SITE_JSON: &str = "projects.json";
/// The undone rows a card names.
pub const NEXT_ROWS: usize = 2;
/// The folder under the tool's local folder where a data repository writes its own card, as
/// `<registered name>.json` (D-24 addendum of 2026-11-25; EP-58b).
pub const CARDS: &str = "cards";
/// The most a card file may weigh; a heavier one is left out unread.
pub const CARD_BYTES: u64 = 64 * 1024;
/// The daily export's two files.
pub const LEDGER_JSON: &str = "ledger.json";
pub const REGISTRY_COPY: &str = "registry.toml";
/// The ends of a project's two CSV files' names (`<project>-sessions.csv`).
const CSV_ENDS: [&str; 2] = ["-sessions.csv", "-snapshots.csv"];
/// How many days of the daily export are kept, today's included, besides each month's first
/// folder, which stays (D-8 addendum of 2026-11-15; EP-41).
pub const KEEP_DAYS: i64 = 14;
/// The pause between two folders a prune removes (rule 9: never a burst of deletions).
pub const PRUNE_PAUSE: Duration = Duration::from_millis(500);

/// The site export: the public projects as of a day.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Site {
    /// The day the figures were read, `YYYY-MM-DD`.
    pub as_of: String,
    pub projects: Vec<SiteProject>,
}

/// One public project's card.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct SiteProject {
    pub name: String,
    /// The stack badge's text: its chips joined, such as `Rust, egui`.
    pub badge: String,
    pub done: usize,
    pub total: usize,
    /// Done of total, rounded down, as the list's progress says it.
    pub percent: u32,
    /// The first undone rows in table order, at most [`NEXT_ROWS`].
    pub next: Vec<NextRow>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NextRow {
    pub id: String,
    /// The Brief cell without its link markup and its decisions.
    pub title: String,
    /// `S`, `M` or `L`; none when the Size cell names none.
    pub size: Option<String>,
}

/// Whether a project's remote is public (EP-7: from the account's list).
pub fn public(hosting: &Hosting) -> bool {
    hosting.class() == Class::Public
}

/// A project's card; `None` for a roadmap without rows.
pub fn site_project(name: &str, chips: &[Chip], roadmap: &Roadmap) -> Option<SiteProject> {
    let counts = roadmap.counts();
    if counts.total == 0 {
        return None;
    }
    let next = roadmap
        .rows()
        .filter(|row| !row.done.is_checked())
        .take(NEXT_ROWS)
        .map(|row| NextRow {
            id: row.id.to_string(),
            title: row.brief.title.clone(),
            size: row.size.map(|size| size.as_str().to_owned()),
        })
        .collect();
    Some(SiteProject {
        name: name.to_owned(),
        badge: chips.iter().map(Chip::text).collect::<Vec<_>>().join(", "),
        done: counts.done,
        total: counts.total,
        percent: u32::try_from(counts.done * 100 / counts.total).unwrap_or(100),
        next,
    })
}

/// `projects.json`: the site as pretty JSON.
pub fn site_json(site: &Site) -> String {
    let mut json = serde_json::to_string_pretty(site).unwrap_or_default();
    json.push('\n');
    json
}

/// A data repository's own card, read from its file and checked: the card and the day its
/// repository wrote it (EP-58b).
#[derive(Debug, Clone, PartialEq)]
pub struct OwnCard {
    pub as_of: NaiveDate,
    pub project: SiteProject,
}

/// A card file as a data repository writes it (D-24 addendum of 2026-11-25): the day and the
/// card's fields; a field it adds besides is ignored, a missing one makes the file malformed.
#[derive(Debug, Deserialize)]
struct CardFile {
    as_of: String,
    name: String,
    badge: String,
    done: usize,
    total: usize,
    percent: u32,
    next: Vec<NextRow>,
}

/// The tool's `cards` folder: `%LOCALAPPDATA%\clauderoadmapwrangler\cards`, or `CRW_HOME\cards`
/// in a trial.
pub fn cards_dir() -> Option<PathBuf> {
    Some(paths::local_dir()?.join(CARDS))
}

/// A data repository's card file in `cards`: `<registered name>.json`.
pub fn card_file(cards: &Path, name: &str) -> PathBuf {
    cards.join(format!("{name}.json"))
}

/// Whether the account's list holds a public repository by one of `names` (a data repository's
/// registered name and its folder's, as the registry writes them), with no git child and no
/// remote read; why it is not public otherwise. An archived public repository is public, as
/// [`public`] counts it.
pub fn listed_public(names: &[&str], account: Option<&[Listed]>) -> Result<(), String> {
    let Some(list) = account else {
        return Err("not public (no account list cached)".to_owned());
    };
    let listed = list.iter().find(|listed| {
        names
            .iter()
            .any(|name| name.eq_ignore_ascii_case(&listed.repo.name))
    });
    match listed.map(|listed| listed.visibility) {
        Some(Visibility::Public) => Ok(()),
        Some(Visibility::Private) => Err("not public (private on the account's list)".to_owned()),
        Some(Visibility::Internal) => Err("not public (internal on the account's list)".to_owned()),
        None => Err("not public (not on the account's list)".to_owned()),
    }
}

/// A data repository's card for the site (EP-58b): its public standing from the account's list
/// by its registered `name` or its `folder`'s name as the registry writes it, then its own file
/// from `cards`, read only when it is public. Nothing inside the repository is read, its folder
/// is not even looked at. Why it is left out otherwise.
pub fn data_card(
    name: &str,
    folder: &str,
    account: Option<&[Listed]>,
    cards: Option<&Path>,
    today: NaiveDate,
) -> Result<OwnCard, String> {
    listed_public(&[name, folder], account)?;
    let cards = cards.ok_or("no profile folder for the cards")?;
    read_card(cards, name, today)
}

/// Reads and checks `<cards>/<name>.json` ([`check_card`]); why it is left out otherwise: no
/// file, a file that does not read or weighs over [`CARD_BYTES`], or one that is malformed.
pub fn read_card(cards: &Path, name: &str, today: NaiveDate) -> Result<OwnCard, String> {
    let path = card_file(cards, name);
    let meta = match fs::metadata(&path) {
        Ok(meta) => meta,
        Err(e) if e.kind() == io::ErrorKind::NotFound => {
            return Err(format!("no card file ({})", path.display()));
        }
        Err(e) => {
            return Err(format!(
                "its card file did not read ({}: {e})",
                path.display()
            ));
        }
    };
    if !meta.is_file() {
        return Err(format!("its card file is not a file ({})", path.display()));
    }
    if meta.len() > CARD_BYTES {
        return Err(format!(
            "its card file is over {} KiB ({})",
            CARD_BYTES / 1024,
            path.display()
        ));
    }
    let text = fs::read_to_string(&path)
        .map_err(|e| format!("its card file did not read ({}: {e})", path.display()))?;
    check_card(&text, name, today).map_err(|why| format!("its card file is malformed: {why}"))
}

/// A card file's text checked (UTF-8, a byte-order mark allowed): JSON with every field, the
/// registered `name` itself, a day written `YYYY-MM-DD` and not after `today`, a total above
/// nought, done at most the total, the percent done of total rounded down, as many next rows as
/// the undone rows allow up to [`NEXT_ROWS`], each with an id, a title and a size of `S`, `M`,
/// `L` or none; why it is malformed otherwise.
pub fn check_card(text: &str, name: &str, today: NaiveDate) -> Result<OwnCard, String> {
    let text = text.strip_prefix('\u{feff}').unwrap_or(text);
    let card: CardFile = serde_json::from_str(text).map_err(|e| e.to_string())?;
    if card.name != name {
        return Err(format!("it names {:?}, not {name:?}", card.name));
    }
    let as_of = NaiveDate::parse_from_str(&card.as_of, "%Y-%m-%d")
        .ok()
        .filter(|day| day.format("%Y-%m-%d").to_string() == card.as_of)
        .ok_or_else(|| format!("its as_of {:?} is no day written YYYY-MM-DD", card.as_of))?;
    if as_of > today {
        return Err(format!("its as_of {as_of} is after today"));
    }
    if card.total == 0 {
        return Err("its total is nought".to_owned());
    }
    if card.done > card.total {
        return Err(format!(
            "done {} is over the total {}",
            card.done, card.total
        ));
    }
    // Checked: a file's counts are not the tool's own and may be any size.
    let percent = card
        .done
        .checked_mul(100)
        .and_then(|hundreds| u32::try_from(hundreds / card.total).ok())
        .ok_or_else(|| format!("its done {} is too large", card.done))?;
    if card.percent != percent {
        return Err(format!(
            "its percent {} is not {} of {} rounded down ({percent})",
            card.percent, card.done, card.total
        ));
    }
    let next = NEXT_ROWS.min(card.total - card.done);
    if card.next.len() != next {
        return Err(format!(
            "it names {} next rows where {} undone call for {next}",
            card.next.len(),
            card.total - card.done
        ));
    }
    for row in &card.next {
        if row.id.trim().is_empty() {
            return Err("a next row has no id".to_owned());
        }
        if row.title.trim().is_empty() {
            return Err(format!("next row {} has no title", row.id));
        }
        if let Some(size) = row.size.as_deref().filter(|s| !["S", "M", "L"].contains(s)) {
            return Err(format!(
                "next row {}'s size {size:?} is not S, M, L or null",
                row.id
            ));
        }
    }
    Ok(OwnCard {
        as_of,
        project: SiteProject {
            name: card.name,
            badge: card.badge,
            done: card.done,
            total: card.total,
            percent,
            next: card.next,
        },
    })
}

/// Text for HTML, in an element or an attribute (the phone's pages).
pub fn escape(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for c in text.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            c => out.push(c),
        }
    }
    out
}

/// The site's `assets\data` folder, joined a component at a time.
pub fn site_data(site_folder: &Path) -> PathBuf {
    SITE_DATA
        .split('/')
        .fold(site_folder.to_path_buf(), |path, part| path.join(part))
}

/// Writes `projects.json` into `<site>/assets/data`, replaced whole, and nothing else in the
/// site's repository (the cards' snippet is no longer written, EP-58b; a copy left there stays
/// untouched); the path written.
pub fn write_site(site_folder: &Path, site: &Site) -> Result<PathBuf, String> {
    let data = site_data(site_folder);
    fs::create_dir_all(&data).map_err(|e| format!("{}: {e}", data.display()))?;
    let path = data.join(SITE_JSON);
    files::write_atomic(&path, site_json(site).as_bytes())
        .map_err(|e| format!("{}: {e}", path.display()))?;
    Ok(path)
}

/// Where the daily export and the CSV go: this repository's git-ignored `private/export`, or
/// `CRW_HOME\export` in a trial.
pub fn folder() -> PathBuf {
    match paths::home() {
        Some(home) => home.join("export"),
        None => paths::repository().join("private").join("export"),
    }
}

/// A day's folder under [`folder`].
pub fn day_folder(folder: &Path, day: NaiveDate) -> PathBuf {
    folder.join(day.format("%Y-%m-%d").to_string())
}

/// Whether a day's export is written: its `ledger.json` is there.
pub fn written(folder: &Path, day: NaiveDate) -> bool {
    day_folder(folder, day).join(LEDGER_JSON).is_file()
}

/// The daily export (D-8): the registry file as it is and every table of the ledger at the
/// schema it holds, into `<folder>/<day>/`, each file replaced whole; the paths written.
pub fn daily(
    registry_file: Option<&Path>,
    ledger: &Ledger,
    folder: &Path,
    day: NaiveDate,
    now: DateTime<Utc>,
) -> Result<Vec<PathBuf>, String> {
    let into = day_folder(folder, day);
    fs::create_dir_all(&into).map_err(|e| format!("{}: {e}", into.display()))?;
    let mut written = Vec::new();
    if let Some(registry) = registry_file.filter(|path| path.is_file()) {
        let bytes = fs::read(registry).map_err(|e| format!("{}: {e}", registry.display()))?;
        let path = into.join(REGISTRY_COPY);
        files::write_atomic(&path, &bytes).map_err(|e| format!("{}: {e}", path.display()))?;
        written.push(path);
    }
    let schema = ledger.schema_version().map_err(|e| e.to_string())?;
    let tables = ledger.dump().map_err(|e| e.to_string())?;
    let document = serde_json::json!({
        "exported_at": now.to_rfc3339_opts(SecondsFormat::Secs, true),
        "schema": schema,
        "tables": tables,
    });
    let mut text = serde_json::to_string_pretty(&document).map_err(|e| e.to_string())?;
    text.push('\n');
    let path = into.join(LEDGER_JSON);
    files::write_atomic(&path, text.as_bytes()).map_err(|e| format!("{}: {e}", path.display()))?;
    written.push(path);
    Ok(written)
}

/// The day a folder under the export folder is for, when its name is a date exactly as
/// [`day_folder`] writes it (`2026-11-15`, not `2026-11-5`).
fn day_of(name: &str) -> Option<NaiveDate> {
    let day = NaiveDate::parse_from_str(name, "%Y-%m-%d").ok()?;
    (day.format("%Y-%m-%d").to_string() == name).then_some(day)
}

/// The dated folders under `folder` a prune on `today` removes, the oldest first: those older
/// than the last [`KEEP_DAYS`] days, today's included, except the earliest folder of each
/// month. A file, or a folder whose name is no such date, is never among them.
pub fn due_for_removal(folder: &Path, today: NaiveDate) -> Vec<PathBuf> {
    let Ok(entries) = fs::read_dir(folder) else {
        return Vec::new();
    };
    let mut days: Vec<(NaiveDate, PathBuf)> = entries
        .flatten()
        .filter_map(|entry| {
            let day = day_of(entry.file_name().to_str()?)?;
            entry.file_type().ok().filter(fs::FileType::is_dir)?;
            Some((day, entry.path()))
        })
        .collect();
    days.sort();
    let oldest_kept = today - TimeDelta::days(KEEP_DAYS - 1);
    let mut months = BTreeSet::new();
    days.into_iter()
        .filter(|(day, _)| {
            // Oldest first, so the first folder met in a month is its earliest.
            let first = months.insert((day.year(), day.month()));
            !first && *day < oldest_kept
        })
        .map(|(_, path)| path)
        .collect()
}

/// Whether a file in a day's folder is one the tool writes there: the daily export's two and a
/// project's CSVs, or one of them half written (`files::write_atomic`'s `.tmp`, left by a write
/// cut short).
fn is_own_file(name: &str) -> bool {
    let name = name.strip_suffix(".tmp").unwrap_or(name);
    name == LEDGER_JSON || name == REGISTRY_COPY || CSV_ENDS.iter().any(|end| name.ends_with(end))
}

/// Removes one dated folder: the tool's own files in it, one at a time, then the folder. A file
/// or a folder the tool did not write keeps the whole folder untouched, and the error says so:
/// the folder is looked through before anything in it is removed.
pub fn remove_day(path: &Path) -> Result<(), String> {
    let entries = fs::read_dir(path).map_err(|e| format!("{}: {e}", path.display()))?;
    let mut own = Vec::new();
    let mut others = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|e| format!("{}: {e}", path.display()))?;
        let name = entry.file_name().to_string_lossy().into_owned();
        let file = entry.file_type().is_ok_and(|kind| kind.is_file());
        if file && is_own_file(&name) {
            own.push(entry.path());
        } else {
            others.push(name);
        }
    }
    if !others.is_empty() {
        others.sort();
        return Err(format!(
            "{} is kept: it holds what the tool did not write there ({})",
            path.display(),
            others.join(", ")
        ));
    }
    for file in own {
        fs::remove_file(&file).map_err(|e| format!("{}: {e}", file.display()))?;
    }
    fs::remove_dir(path).map_err(|e| format!("{}: {e}", path.display()))
}

/// The daily export's keep (D-8 addendum of 2026-11-15; EP-41), run after each write: removes
/// [`due_for_removal`]'s folders one at a time, `pause` between two, so a backlog after a long
/// absence is never a burst (rule 9); nothing else under `folder` is touched. The folders
/// removed, and why each other one was not.
pub fn prune(folder: &Path, today: NaiveDate, pause: Duration) -> (Vec<PathBuf>, Vec<String>) {
    let mut removed = Vec::new();
    let mut failed = Vec::new();
    for (n, path) in due_for_removal(folder, today).into_iter().enumerate() {
        if n > 0 {
            std::thread::sleep(pause);
        }
        match remove_day(&path) {
            Ok(()) => removed.push(path),
            Err(why) => failed.push(why),
        }
    }
    (removed, failed)
}

/// The CSV columns of a project's sessions.
pub const SESSION_COLUMNS: [&str; 16] = [
    "session_id",
    "ep",
    "mode",
    "model",
    "effort",
    "ultracode",
    "started_at",
    "ended_at",
    "minutes",
    "outcome",
    "cost_usd",
    "questions",
    "answer_latency_s",
    "context_percent",
    "weekly_share",
    "fable_share",
];

/// A project's sessions as CSV (RFC 4180, CRLF), a header line first; the run's text and its
/// start's record stay in the daily export.
pub fn sessions_csv(rows: &[SessionRow]) -> String {
    let number = |value: Option<f64>| value.map(|v| v.to_string()).unwrap_or_default();
    let mut csv = line(SESSION_COLUMNS.iter().map(|c| (*c).to_owned()));
    for row in rows {
        csv.push_str(&line([
            row.session_id.clone().unwrap_or_default(),
            row.ep.clone().unwrap_or_default(),
            row.mode.clone(),
            row.model.clone().unwrap_or_default(),
            row.effort.clone().unwrap_or_default(),
            row.ultracode.to_string(),
            row.started_at.clone(),
            row.ended_at.clone().unwrap_or_default(),
            row.took()
                .map(|took| took.num_minutes().to_string())
                .unwrap_or_default(),
            row.outcome.clone().unwrap_or_default(),
            number(row.cost_usd),
            row.questions.to_string(),
            number(row.answer_latency_s),
            number(row.context_percent),
            number(row.usage.weekly_share),
            number(row.usage.fable_share),
        ]));
    }
    csv
}

/// A project's daily counts as CSV: when, done, total.
pub fn snapshots_csv(rows: &[SnapshotRow]) -> String {
    let mut csv = line(["taken_at", "done", "total"].map(str::to_owned));
    for row in rows {
        csv.push_str(&line([
            row.taken_at.to_rfc3339_opts(SecondsFormat::Secs, true),
            row.done.to_string(),
            row.total.to_string(),
        ]));
    }
    csv
}

fn line(fields: impl IntoIterator<Item = String>) -> String {
    let mut out = fields
        .into_iter()
        .map(|field| field_csv(&field))
        .collect::<Vec<_>>()
        .join(",");
    out.push_str("\r\n");
    out
}

/// A field quoted when it holds a comma, a quote or a line break, its quotes doubled.
fn field_csv(field: &str) -> String {
    if field.contains([',', '"', '\n', '\r']) {
        format!("\"{}\"", field.replace('"', "\"\""))
    } else {
        field.to_owned()
    }
}

/// Writes a project's two CSV files into `<folder>/<day>/`; the paths written.
pub fn write_csv(
    project: &str,
    sessions: &[SessionRow],
    snapshots: &[SnapshotRow],
    folder: &Path,
    day: NaiveDate,
) -> Result<Vec<PathBuf>, String> {
    let into = day_folder(folder, day);
    fs::create_dir_all(&into).map_err(|e| format!("{}: {e}", into.display()))?;
    let mut written = Vec::new();
    for (suffix, text) in [
        ("sessions", sessions_csv(sessions)),
        ("snapshots", snapshots_csv(snapshots)),
    ] {
        let path = into.join(format!("{project}-{suffix}.csv"));
        files::write_atomic(&path, text.as_bytes())
            .map_err(|e| format!("{}: {e}", path.display()))?;
        written.push(path);
    }
    Ok(written)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::badges::hosting::{self, Listed};
    use crate::roadmap::table;

    const README: &str = "\
## Phase P1 — Start

| # | Brief | Size | Depends on | Core | Done |
|---|---|---|---|---|---|
| EP-1 | [Bootstrap](EP-1-bootstrap.md) (D-1) | S | — | core | ☑ `abc1234` |
| EP-2 | Registry & <discovery> | M | EP-1 | core | ☐ |
| EP-3 | Parser \"A\" | L | EP-2 | core | ☐ |
| EP-4 | Parser B | M | EP-3 | core | ☐ |
";

    fn temp(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("crw-export-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn site() -> Site {
        let roadmap = table::parse(README);
        Site {
            as_of: "2026-11-13".to_owned(),
            projects: vec![site_project("harbor", &[], &roadmap).unwrap()],
        }
    }

    #[test]
    fn a_card_counts_and_names_the_next_two_undone_rows() {
        let roadmap = table::parse(README);
        let project = site_project("harbor", &[], &roadmap).unwrap();
        assert_eq!((project.done, project.total, project.percent), (1, 4, 25));
        let ids: Vec<&str> = project.next.iter().map(|row| row.id.as_str()).collect();
        assert_eq!(
            ids,
            ["EP-2", "EP-3"],
            "the checked row skipped, two at most"
        );
        assert_eq!(project.next[0].title, "Registry & <discovery>");
        assert_eq!(project.next[0].size.as_deref(), Some("M"));
        assert!(site_project("empty", &[], &table::parse("# Nothing\n")).is_none());
    }

    #[test]
    fn the_json_holds_the_fields_the_site_reads() {
        let json: serde_json::Value = serde_json::from_str(&site_json(&site())).unwrap();
        assert_eq!(json["as_of"], "2026-11-13");
        let project = &json["projects"][0];
        for field in ["name", "badge", "done", "total", "percent", "next"] {
            assert!(project.get(field).is_some(), "{field} missing");
        }
        assert_eq!(project["next"][1]["title"], "Parser \"A\"");
    }

    #[test]
    fn escape_covers_the_five_characters() {
        assert_eq!(
            escape("Registry & <discovery> \"A\" 'b'"),
            "Registry &amp; &lt;discovery&gt; &quot;A&quot; &#39;b&#39;"
        );
    }

    /// A synthetic card file's text, as a data repository's hook writes it.
    const CARD: &str = r#"{
  "as_of": "2026-11-20",
  "name": "quarry",
  "badge": "Python, DuckDB",
  "done": 7,
  "total": 10,
  "percent": 70,
  "next": [
    {"id": "EP-8", "title": "Loader", "size": "S"},
    {"id": "EP-9", "title": "Views & checks", "size": null}
  ],
  "written_by": "a field the reader ignores"
}
"#;

    fn today() -> NaiveDate {
        NaiveDate::from_ymd_opt(2026, 11, 25).unwrap()
    }

    #[test]
    fn a_card_file_reads_into_a_site_card_and_its_day() {
        let card = check_card(CARD, "quarry", today()).unwrap();
        assert_eq!(card.as_of, NaiveDate::from_ymd_opt(2026, 11, 20).unwrap());
        assert_eq!(card.project.name, "quarry");
        assert_eq!(card.project.badge, "Python, DuckDB");
        assert_eq!(
            (card.project.done, card.project.total, card.project.percent),
            (7, 10, 70)
        );
        assert_eq!(card.project.next[1].size, None);
        // A byte-order mark is allowed; a card of today is not after today.
        let marked = format!("\u{feff}{}", CARD.replace("2026-11-20", "2026-11-25"));
        assert!(check_card(&marked, "quarry", today()).is_ok());
        // A done roadmap names no next row.
        let done = CARD
            .replace("\"done\": 7", "\"done\": 10")
            .replace("\"percent\": 70", "\"percent\": 100")
            .replace(
                r#"{"id": "EP-8", "title": "Loader", "size": "S"},
    {"id": "EP-9", "title": "Views & checks", "size": null}"#,
                "",
            );
        assert!(check_card(&done, "quarry", today()).is_ok(), "{done}");
        // One undone row, one next row; a size left out is none.
        let one = CARD
            .replace("\"done\": 7", "\"done\": 9")
            .replace("\"percent\": 70", "\"percent\": 90")
            .replace(
                r#"{"id": "EP-8", "title": "Loader", "size": "S"},
    {"id": "EP-9", "title": "Views & checks", "size": null}"#,
                r#"{"id": "EP-10", "title": "Last"}"#,
            );
        let card = check_card(&one, "quarry", today()).unwrap();
        assert_eq!(card.project.next[0].size, None);
    }

    #[test]
    fn a_malformed_card_file_says_why() {
        let why = |text: &str| check_card(text, "quarry", today()).unwrap_err();
        assert!(why("not json").contains("expected"), "{}", why("not json"));
        assert!(why(&CARD.replace("\"badge\": \"Python, DuckDB\",", "")).contains("badge"));
        assert!(why(&CARD.replace("\"quarry\"", "\"Quarry\"")).contains("not \"quarry\""));
        assert!(why(&CARD.replace("2026-11-20", "2026-11-5")).contains("YYYY-MM-DD"));
        assert!(why(&CARD.replace("2026-11-20", "20 Nov")).contains("YYYY-MM-DD"));
        assert!(why(&CARD.replace("2026-11-20", "2026-11-26")).contains("after today"));
        assert!(why(&CARD.replace("\"total\": 10", "\"total\": 0")).contains("nought"));
        assert!(why(&CARD.replace("\"done\": 7", "\"done\": 11")).contains("over the total"));
        assert!(why(&CARD.replace("\"percent\": 70", "\"percent\": 71")).contains("rounded down"));
        let huge = CARD
            .replace("\"done\": 7", "\"done\": 18446744073709551615")
            .replace("\"total\": 10", "\"total\": 18446744073709551615");
        assert!(why(&huge).contains("too large"), "{}", why(&huge));
        let three = CARD.replace(
            r#"{"id": "EP-8", "title": "Loader", "size": "S"},"#,
            r#"{"id": "EP-7", "title": "Extra", "size": "S"}, {"id": "EP-8", "title": "Loader", "size": "S"},"#,
        );
        assert!(why(&three).contains("3 next rows"), "{}", why(&three));
        let one = CARD.replace(r#"{"id": "EP-8", "title": "Loader", "size": "S"},"#, "");
        assert!(why(&one).contains("1 next rows"), "{}", why(&one));
        assert!(why(&CARD.replace("\"size\": \"S\"", "\"size\": \"XL\"")).contains("not S, M, L"));
        assert!(why(&CARD.replace("\"id\": \"EP-8\"", "\"id\": \" \"")).contains("no id"));
        assert!(why(&CARD.replace("\"Loader\"", "\"\"")).contains("no title"));
    }

    fn account(visibility: &str) -> Vec<Listed> {
        hosting::parse_account(&format!(
            r#"[{{"name": "quarry-data", "owner": {{"login": "someone"}},
                 "visibility": "{visibility}", "isArchived": false,
                 "pushedAt": "2026-11-01T10:00:00Z"}}]"#
        ))
        .unwrap()
    }

    #[test]
    fn a_data_repositorys_standing_comes_from_the_list_by_its_names() {
        let open = account("PUBLIC");
        assert!(listed_public(&["quarry", "quarry-data"], Some(&open)).is_ok());
        assert!(
            listed_public(&["Quarry-Data"], Some(&open)).is_ok(),
            "case-blind"
        );
        let why = |names: &[&str], list: Option<&[Listed]>| listed_public(names, list).unwrap_err();
        assert!(why(&["quarry"], Some(&open)).contains("not on the account's list"));
        assert!(why(&["quarry-data"], Some(&account("PRIVATE"))).contains("private"));
        assert!(why(&["quarry-data"], Some(&account("INTERNAL"))).contains("internal"));
        assert!(why(&["quarry-data"], None).contains("no account list"));
    }

    #[test]
    fn a_data_card_is_read_from_its_file_only_when_public() {
        let dir = temp("cards");
        let cards = dir.join(CARDS);
        let open = account("PUBLIC");
        // No file yet: left out, the path named.
        let why =
            data_card("quarry", "quarry-data", Some(&open), Some(&cards), today()).unwrap_err();
        assert!(why.starts_with("no card file"), "{why}");
        assert!(why.contains("quarry.json"), "{why}");
        fs::create_dir_all(&cards).unwrap();
        fs::write(card_file(&cards, "quarry"), CARD).unwrap();
        let card = data_card("quarry", "quarry-data", Some(&open), Some(&cards), today()).unwrap();
        assert_eq!(card.project.total, 10);
        // Private: the file is never opened, so even a broken one is left out as not public.
        fs::write(card_file(&cards, "quarry"), "{").unwrap();
        let why = data_card(
            "quarry",
            "quarry-data",
            Some(&account("PRIVATE")),
            Some(&cards),
            today(),
        )
        .unwrap_err();
        assert!(why.starts_with("not public"), "{why}");
        let why =
            data_card("quarry", "quarry-data", Some(&open), Some(&cards), today()).unwrap_err();
        assert!(why.starts_with("its card file is malformed"), "{why}");
        // Too heavy: left out unread.
        fs::write(
            card_file(&cards, "quarry"),
            vec![b' '; usize::try_from(CARD_BYTES).unwrap() + 1],
        )
        .unwrap();
        let why =
            data_card("quarry", "quarry-data", Some(&open), Some(&cards), today()).unwrap_err();
        assert!(why.contains("over 64 KiB"), "{why}");
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn only_a_public_remote_is_public() {
        let listed = |visibility: &str| -> Vec<Listed> {
            hosting::parse_account(&format!(
                r#"[{{"name": "harbor", "owner": {{"login": "someone"}},
                     "visibility": "{visibility}", "isArchived": false,
                     "pushedAt": "2026-11-01T10:00:00Z"}}]"#
            ))
            .unwrap()
        };
        let remote = Some("https://github.com/someone/harbor.git");
        let (open, closed) = (listed("PUBLIC"), listed("PRIVATE"));
        assert!(public(&hosting::classify(
            &["harbor"],
            remote,
            Some(&open[..])
        )));
        assert!(!public(&hosting::classify(
            &["harbor"],
            remote,
            Some(&closed[..])
        )));
        assert!(
            !public(&hosting::classify(&["harbor"], remote, None)),
            "no list, not public"
        );
        assert!(
            !public(&hosting::classify(&["harbor"], None, Some(&open[..]))),
            "an orphan"
        );
    }

    #[test]
    fn csv_fields_are_quoted_when_they_must_be() {
        assert_eq!(field_csv("plain"), "plain");
        assert_eq!(field_csv("a, b"), "\"a, b\"");
        assert_eq!(field_csv("say \"hi\""), "\"say \"\"hi\"\"\"");
        assert_eq!(field_csv("two\nlines"), "\"two\nlines\"");
        let rows = [SnapshotRow {
            taken_at: "2026-11-12T10:00:00Z".parse().unwrap(),
            done: 3,
            total: 9,
        }];
        assert_eq!(
            snapshots_csv(&rows),
            "taken_at,done,total\r\n2026-11-12T10:00:00Z,3,9\r\n"
        );
        let header = sessions_csv(&[]);
        assert_eq!(header.trim_end().split(',').count(), SESSION_COLUMNS.len());
    }

    #[test]
    fn the_daily_export_writes_the_registry_and_every_table() {
        let dir = temp("daily");
        let ledger = Ledger::open(&dir.join("ledger.sqlite")).unwrap();
        ledger
            .record_counts(
                "harbor",
                "2026-11-12T10:00:00Z".parse().unwrap(),
                3,
                9,
                &chrono::Utc,
            )
            .unwrap();
        let registry = dir.join("registry.toml");
        fs::write(&registry, "version = 1\n").unwrap();
        let day = NaiveDate::from_ymd_opt(2026, 11, 13).unwrap();
        let out = dir.join("export");
        assert!(!written(&out, day));
        let paths = daily(
            Some(&registry),
            &ledger,
            &out,
            day,
            "2026-11-13T12:00:00Z".parse().unwrap(),
        )
        .unwrap();
        assert_eq!(paths.len(), 2);
        assert!(written(&out, day));
        let copy = fs::read_to_string(day_folder(&out, day).join(REGISTRY_COPY)).unwrap();
        assert_eq!(copy, "version = 1\n");
        let json: serde_json::Value = serde_json::from_str(
            &fs::read_to_string(day_folder(&out, day).join(LEDGER_JSON)).unwrap(),
        )
        .unwrap();
        assert_eq!(json["schema"], crate::ledger::SCHEMA_VERSION);
        assert_eq!(json["tables"]["snapshot"][0]["project"], "harbor");
        assert_eq!(json["tables"]["snapshot"][0]["total"], 9);
        assert!(json["tables"]["session"].is_array());
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn the_keep_holds_fourteen_days_and_each_months_first_and_touches_nothing_else() {
        let dir = temp("prune");
        let out = dir.join("export");
        let day = |month, day| NaiveDate::from_ymd_opt(2026, month, day).unwrap();
        let make = |date: NaiveDate| {
            let into = day_folder(&out, date);
            fs::create_dir_all(&into).unwrap();
            fs::write(into.join(LEDGER_JSON), "{}").unwrap();
            fs::write(into.join(REGISTRY_COPY), "version = 1\n").unwrap();
        };
        // October from the 20th, November but the 14th, December's first days.
        for d in 20..=31 {
            make(day(10, d));
        }
        for d in (1..=30).filter(|&d| d != 14) {
            make(day(11, d));
        }
        for d in 1..=3 {
            make(day(12, d));
        }
        // A CSV and a half-written file are the tool's; a note of the owner's keeps its folder
        // whole; other names stay.
        fs::write(day_folder(&out, day(11, 4)).join("ledger.json.tmp"), "half").unwrap();
        fs::write(
            day_folder(&out, day(11, 2)).join("harbor-sessions.csv"),
            "x",
        )
        .unwrap();
        fs::write(day_folder(&out, day(11, 3)).join("notes.txt"), "mine").unwrap();
        fs::create_dir_all(out.join("2026-11-5")).unwrap();
        fs::create_dir_all(out.join("archive")).unwrap();
        fs::write(out.join("2026-11-14"), "a file named as a date").unwrap();

        let today = day(12, 3);
        // Fourteen days kept, today's included: from 2026-11-20; each month's first too.
        let due = due_for_removal(&out, today);
        assert!(
            !due.contains(&day_folder(&out, day(10, 20))),
            "October's first"
        );
        assert!(
            !due.contains(&day_folder(&out, day(11, 1))),
            "November's first"
        );
        assert!(
            !due.contains(&day_folder(&out, day(11, 20))),
            "the oldest kept day"
        );
        assert!(due.contains(&day_folder(&out, day(11, 19))));
        assert!(due.contains(&day_folder(&out, day(10, 21))));
        assert_eq!(
            due.len(),
            11 + 17,
            "October 21-31 and November 2-19 but the 14th"
        );
        let (removed, failed) = prune(&out, today, Duration::ZERO);
        assert_eq!(removed.len(), due.len() - 1);
        assert_eq!(failed.len(), 1, "{failed:?}");
        assert!(failed[0].contains("notes.txt"), "{failed:?}");
        assert!(
            !day_folder(&out, day(11, 2)).exists(),
            "its CSV went with it"
        );
        assert!(
            !day_folder(&out, day(11, 4)).exists(),
            "a half-written file is the tool's"
        );
        assert!(day_folder(&out, day(11, 3)).join("notes.txt").is_file());
        assert!(
            written(&out, day(11, 3)),
            "a folder holding the owner's note is left untouched"
        );
        assert!(out.join("2026-11-5").is_dir() && out.join("archive").is_dir());
        assert!(out.join("2026-11-14").is_file());
        for kept in [
            day(10, 20),
            day(11, 1),
            day(11, 20),
            day(11, 30),
            day(12, 1),
            day(12, 3),
        ] {
            assert!(written(&out, kept), "{kept} is kept");
        }
        // Run again the same day: only the folder the owner's note keeps is due, and it stays.
        let (removed, failed) = prune(&out, today, Duration::ZERO);
        assert!(removed.is_empty());
        assert_eq!(failed.len(), 1);
        // No export folder at all: nothing to do.
        assert_eq!(
            prune(&dir.join("none"), today, Duration::ZERO),
            (Vec::new(), Vec::new())
        );
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn the_site_file_goes_into_assets_data_alone() {
        let dir = temp("site");
        let data = dir.join("assets").join("data");
        // A snippet an older export wrote stays as it is.
        fs::create_dir_all(&data).unwrap();
        fs::write(data.join("projects-cards.html"), "old").unwrap();
        let path = write_site(&dir, &site()).unwrap();
        assert_eq!(path, data.join(SITE_JSON));
        let names: Vec<_> = fs::read_dir(&dir)
            .unwrap()
            .flatten()
            .map(|e| e.file_name())
            .collect();
        assert_eq!(names, ["assets"]);
        let mut written: Vec<_> = fs::read_dir(&data)
            .unwrap()
            .flatten()
            .map(|e| e.file_name())
            .collect();
        written.sort();
        assert_eq!(written, ["projects-cards.html", SITE_JSON]);
        assert_eq!(
            fs::read_to_string(data.join("projects-cards.html")).unwrap(),
            "old"
        );
        let _ = fs::remove_dir_all(&dir);
    }
}
