//! The badges as the window draws them (D-24, D-25): each reading becomes a [`Mark`], an icon or a
//! few characters in a tone with the whole sentence as hover text, for the row's badge column and
//! the Detail pane's Status section. Every mark is made from a [`Sign`] of the tables below, and the
//! legend under the Projects pane lists the same tables, so the key cannot drift from the row
//! (EP-7a).

use chrono::{DateTime, Local, Utc};
use crw_core::badges::activity::Staleness;
use crw_core::badges::backup::State as BackupState;
use crw_core::badges::health::Outcome;
use crw_core::badges::hosting::{Class, Hosting, Verdict, Visibility};
use crw_core::badges::short_age;
use crw_core::badges::stack::Kind;
use crw_core::ledger;
use eframe::egui::{self, Align, Color32, Layout, RichText, vec2};
use egui_material_icons::MaterialIcon;

use crate::portfolio::ProjectRow;
use crate::style::{self, icons};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tone {
    Good,
    Plain,
    Weak,
    Warn,
    Bad,
}

impl Tone {
    pub fn colour(self, ui: &egui::Ui) -> Color32 {
        match self {
            Tone::Good => style::green(ui),
            Tone::Plain => ui.visuals().text_color(),
            Tone::Weak => ui.visuals().weak_text_color(),
            Tone::Warn => style::amber(ui),
            Tone::Bad => style::red(ui),
        }
    }
}

/// What a sign looks like: an icon of the symbol font, or a few characters.
#[derive(Clone, Copy, Debug)]
pub enum Glyph {
    Icon(MaterialIcon),
    Text(&'static str),
}

/// Two glyphs are the same when they draw the same: an icon by its code point and style.
impl PartialEq for Glyph {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Glyph::Icon(a), Glyph::Icon(b)) => a.codepoint == b.codepoint && a.style == b.style,
            (Glyph::Text(a), Glyph::Text(b)) => a == b,
            _ => false,
        }
    }
}

/// One entry of the key: what a mark looks like, its tone, and what it means in a few words.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Sign {
    pub glyph: Glyph,
    pub tone: Tone,
    pub meaning: &'static str,
}

impl Sign {
    const fn icon(icon: MaterialIcon, tone: Tone, meaning: &'static str) -> Self {
        Self {
            glyph: Glyph::Icon(icon),
            tone,
            meaning,
        }
    }

    const fn text(text: &'static str, tone: Tone, meaning: &'static str) -> Self {
        Self {
            glyph: Glyph::Text(text),
            tone,
            meaning,
        }
    }

    /// The glyph in its tone's colour.
    pub fn rich(&self, ui: &egui::Ui) -> RichText {
        let colour = self.tone.colour(ui);
        match self.glyph {
            Glyph::Icon(icon) => style::icon(icon, Some(colour)),
            Glyph::Text(text) => RichText::new(text).color(colour),
        }
    }

    /// Draws the sign with a sentence of its own as hover text.
    pub fn show(&self, ui: &mut egui::Ui, hover: &str) -> egui::Response {
        ui.label(self.rich(ui)).on_hover_text(hover)
    }
}

// Hosting (D-24): where the repository lives.
const PUBLIC: Sign = Sign::icon(icons::ICON_PUBLIC, Tone::Good, "public on GitHub");
const PRIVATE: Sign = Sign::icon(icons::ICON_LOCK, Tone::Plain, "private on GitHub");
const ARCHIVED: Sign = Sign::icon(icons::ICON_INVENTORY_2, Tone::Weak, "archived on GitHub");
const UNLISTED: Sign = Sign::icon(icons::ICON_CLOUD, Tone::Weak, "GitHub, list not read");
const ELSEWHERE: Sign = Sign::icon(icons::ICON_DNS, Tone::Plain, "a remote elsewhere");
const LOCAL: Sign = Sign::icon(icons::ICON_LAPTOP, Tone::Weak, "local only");
const ORPHAN: Sign = Sign::icon(icons::ICON_LINK_OFF, Tone::Warn, "orphan on GitHub");
const MISSING: Sign = Sign::icon(
    icons::ICON_CLOUD_OFF,
    Tone::Bad,
    "origin not on the account",
);

// The clone against its origin: the head check, else git's counts as last fetched (D-24).
const IN_SYNC: Sign = Sign::text("=", Tone::Weak, "in sync");
const AHEAD: Sign = Sign::text("↑n", Tone::Warn, "n not pushed");
const BEHIND: Sign = Sign::text("↓n", Tone::Warn, "n not pulled");
const DIVERGED: Sign = Sign::text("↑n↓n", Tone::Bad, "diverged");
const NOT_FETCHED: Sign = Sign::icon(
    icons::ICON_SYNC_PROBLEM,
    Tone::Warn,
    "commits never fetched",
);
const NO_BRANCH: Sign = Sign::text("?", Tone::Weak, "upstream gone");
const NO_UPSTREAM: Sign = Sign::text("–", Tone::Weak, "no upstream");
const ORIGIN_UNREAD: Sign = Sign::icon(icons::ICON_SYNC_PROBLEM, Tone::Weak, "origin not asked");

// The last commit's age in its staleness colour (D-6's 60 days), a dot for a dirty tree.
const FRESH: Sign = Sign::text("3 d", Tone::Plain, "under 7 days");
const QUIET: Sign = Sign::text("3 w", Tone::Weak, "under 60 days");
const STALE: Sign = Sign::text("4 mo", Tone::Warn, "60 days or more");
const NO_COMMIT: Sign = Sign::text("new", Tone::Weak, "no commit yet");
const DIRTY: Sign = Sign::text("●", Tone::Warn, "uncommitted changes");

// CI and license (D-25's health).
const PASSED: Sign = Sign::icon(icons::ICON_CHECK_CIRCLE, Tone::Good, "latest run passed");
const FAILED: Sign = Sign::icon(icons::ICON_CANCEL, Tone::Bad, "latest run failed");
const RUNNING: Sign = Sign::icon(icons::ICON_PENDING, Tone::Warn, "a run in progress");
const OTHER_RUN: Sign = Sign::icon(icons::ICON_HELP, Tone::Weak, "cancelled, or no run");
const RUN_UNASKED: Sign = Sign::icon(icons::ICON_HOURGLASS_EMPTY, Tone::Weak, "GitHub not asked");
const RUN_UNREAD: Sign = Sign::icon(icons::ICON_SYNC_PROBLEM, Tone::Weak, "GitHub not reached");
const NO_LICENSE: Sign = Sign::icon(icons::ICON_GAVEL, Tone::Warn, "public, no LICENSE");

// Backup (D-24): the Documents job, which runs daily.
const BACKED_UP: Sign = Sign::icon(icons::ICON_BACKUP, Tone::Good, "within 48 h");
const BACKUP_STALE: Sign = Sign::icon(icons::ICON_BACKUP, Tone::Warn, "not within 48 h");
const BACKUP_FAILING: Sign = Sign::icon(icons::ICON_ERROR, Tone::Bad, "the job reported a problem");
const NOT_COVERED: Sign = Sign::icon(icons::ICON_CLOUD_OFF, Tone::Warn, "not in the job");
const BACKUP_UNKNOWN: Sign = Sign::icon(icons::ICON_HELP, Tone::Weak, "no archive yet");

// Usage (D-23, D-25): the project's share of the week, in points of the weekly limit.
const USED: Sign = Sign::text("+2 %", Tone::Plain, "points of the weekly limit this week");
const UNMOVED: Sign = Sign::text("+0 %", Tone::Weak, "worked, no rise yet");

// The row's other marks.
pub const PINNED: Sign = Sign::icon(icons::ICON_PUSH_PIN, Tone::Weak, "pinned");
pub const PROPOSAL: Sign = Sign::icon(icons::ICON_LIGHTBULB, Tone::Warn, "state change proposed");
pub const SESSION: Sign = Sign::icon(icons::ICON_EDIT_NOTE, Tone::Warn, "session in progress");
pub const GIT_PENDING: Sign = Sign::icon(icons::ICON_SCHEDULE, Tone::Weak, "git not asked yet");
pub const OWNER_ITEMS: Sign = Sign::icon(icons::ICON_TASK_ALT, Tone::Warn, "open owner items");
pub const MESSAGES: Sign = Sign::icon(icons::ICON_INBOX, Tone::Warn, "open inbox messages");
pub const URGENT: Sign = Sign::icon(icons::ICON_INBOX, Tone::Bad, "an urgent message");
pub const FOLDER: Sign = Sign::icon(icons::ICON_FOLDER, Tone::Weak, "folder not adopted");

/// A group of the key, named after its column.
pub struct Group {
    pub title: &'static str,
    pub signs: &'static [Sign],
}

/// Every sign a mark is made from, in the badge column's order and then the row's other marks;
/// the legend shows the stack chips before them.
pub const KEY: [Group; 7] = [
    Group {
        title: "Hosting",
        signs: &[
            PUBLIC, PRIVATE, ARCHIVED, UNLISTED, ELSEWHERE, LOCAL, ORPHAN, MISSING,
        ],
    },
    Group {
        title: "Sync",
        signs: &[
            IN_SYNC,
            AHEAD,
            BEHIND,
            DIVERGED,
            NOT_FETCHED,
            NO_BRANCH,
            NO_UPSTREAM,
            ORIGIN_UNREAD,
        ],
    },
    Group {
        title: "Age",
        signs: &[FRESH, QUIET, STALE, NO_COMMIT, DIRTY],
    },
    Group {
        title: "CI and license",
        signs: &[
            PASSED,
            FAILED,
            RUNNING,
            OTHER_RUN,
            RUN_UNASKED,
            RUN_UNREAD,
            NO_LICENSE,
        ],
    },
    Group {
        title: "Backup",
        signs: &[
            BACKED_UP,
            BACKUP_STALE,
            BACKUP_FAILING,
            NOT_COVERED,
            BACKUP_UNKNOWN,
        ],
    },
    Group {
        title: "Usage",
        signs: &[USED, UNMOVED],
    },
    Group {
        title: "Row",
        signs: &[
            PINNED,
            PROPOSAL,
            SESSION,
            GIT_PENDING,
            OWNER_ITEMS,
            MESSAGES,
            URGENT,
            FOLDER,
        ],
    },
];

/// One badge: its sign, the characters it shows when they carry a count or an age, and the
/// sentence behind it.
#[derive(Clone, Debug)]
pub struct Mark {
    pub sign: Sign,
    /// Drawn instead of the sign's own sample: `↑3` for `↑n`, `5 h` for an age.
    pub text: Option<String>,
    pub hover: String,
}

impl Mark {
    fn new(sign: Sign, hover: impl Into<String>) -> Self {
        Self {
            sign,
            text: None,
            hover: hover.into(),
        }
    }

    /// The sign with each `n` of its sample replaced by the next count: `↑n↓n` → `↑2↓5`.
    fn counted(sign: Sign, counts: &[u32], hover: impl Into<String>) -> Self {
        let Glyph::Text(sample) = sign.glyph else {
            return Self::new(sign, hover);
        };
        let mut counts = counts.iter();
        let text = sample
            .chars()
            .map(|c| match c {
                'n' => counts.next().map_or_else(|| c.to_string(), u32::to_string),
                c => c.to_string(),
            })
            .collect();
        Self {
            sign,
            text: Some(text),
            hover: hover.into(),
        }
    }

    /// The sign's tone on other characters: an age.
    fn worded(sign: Sign, text: impl Into<String>, hover: impl Into<String>) -> Self {
        Self {
            sign,
            text: Some(text.into()),
            hover: hover.into(),
        }
    }

    /// The mark as text in its tone's colour.
    pub fn rich(&self, ui: &egui::Ui) -> RichText {
        match &self.text {
            Some(text) => RichText::new(text).color(self.sign.tone.colour(ui)),
            None => self.sign.rich(ui),
        }
    }

    /// Draws the mark where the layout puts it, with its hover text.
    pub fn show(&self, ui: &mut egui::Ui) -> egui::Response {
        ui.label(self.rich(ui)).on_hover_text(&self.hover)
    }
}

/// `1 commit`, `3 commits`.
fn commits(n: u32) -> String {
    match n {
        1 => "1 commit".to_owned(),
        n => format!("{n} commits"),
    }
}

fn ago(now: DateTime<Utc>, at: DateTime<Utc>) -> String {
    match short_age(now - at).as_str() {
        "now" => "just now".to_owned(),
        age => format!("{age} ago"),
    }
}

/// Where the repository lives.
pub fn hosting(row: &ProjectRow, now: DateTime<Utc>) -> Mark {
    match &row.badges.hosting {
        Hosting::Listed(listed) => {
            let (sign, word) = match (listed.archived, listed.visibility) {
                (true, _) => (ARCHIVED, "Archived"),
                (false, Visibility::Public) => (PUBLIC, "Public"),
                (false, Visibility::Private) => (PRIVATE, "Private"),
                (false, Visibility::Internal) => (PRIVATE, "Internal"),
            };
            let mut hover = format!("{word} on GitHub: {}", listed.repo.slug());
            if let Some(at) = listed.pushed_at {
                hover.push_str(&format!(", pushed {}", ago(now, at)));
            }
            if let Some(branch) = &listed.default_branch {
                hover.push_str(&format!(", default branch {branch}"));
            }
            Mark::new(sign, hover)
        }
        Hosting::Local => Mark::new(LOCAL, "Local only: no remote"),
        Hosting::Orphan(listed) => Mark::new(
            ORPHAN,
            format!(
                "Orphan: no remote here, but the account holds {}{}",
                listed.repo.slug(),
                listed
                    .pushed_at
                    .map(|at| format!(", last pushed {}", ago(now, at)))
                    .unwrap_or_default()
            ),
        ),
        Hosting::Missing(repo) => Mark::new(
            MISSING,
            format!(
                "Missing: the origin names {}, which the account does not list",
                repo.slug()
            ),
        ),
        Hosting::GitHub(repo) => Mark::new(
            UNLISTED,
            format!(
                "On GitHub: {} (not in the account's list, or the list not read yet)",
                repo.slug()
            ),
        ),
        Hosting::Elsewhere(host) => Mark::new(ELSEWHERE, format!("A remote at {host}")),
    }
}

/// Whether the clone and its origin hold the same commits: the head check while it still
/// describes the clone, else git's own counts against the upstream as last fetched.
pub fn sync(row: &ProjectRow, now: DateTime<Utc>) -> Option<Mark> {
    let badges = &row.badges;
    badges.remote.as_ref()?;
    if let Some(head) = badges.head.as_ref().filter(|_| badges.head_is_current()) {
        let check = &head.value;
        let checked = format!("the origin checked {}", ago(now, head.at));
        return Some(match &check.verdict {
            Verdict::InSync => Mark::new(
                IN_SYNC,
                format!("In sync: {checked} ({})", check.remote_ref),
            ),
            Verdict::Ahead(n) => Mark::counted(
                AHEAD,
                &[*n],
                format!("{} not pushed: {checked}", commits(*n)),
            ),
            Verdict::Behind(n) => Mark::counted(
                BEHIND,
                &[*n],
                format!("{} on the origin not pulled: {checked}", commits(*n)),
            ),
            Verdict::Diverged { ahead, behind } => Mark::counted(
                DIVERGED,
                &[*ahead, *behind],
                format!("Diverged, {ahead} ahead and {behind} behind: {checked}"),
            ),
            Verdict::NotFetched => Mark::new(
                NOT_FETCHED,
                format!(
                    "The origin holds commits this clone has never fetched, so it is behind or \
                     diverged: {checked}"
                ),
            ),
            Verdict::NoBranch => Mark::new(
                NO_BRANCH,
                format!("The origin has no {}: {checked}", check.remote_ref),
            ),
            Verdict::Failed(reason) => Mark::new(
                ORIGIN_UNREAD,
                format!("The origin could not be asked ({checked}): {reason}"),
            ),
        });
    }
    let activity = badges.activity.as_ref()?;
    let fetched = |what: String| {
        format!("{what} as git last saw the upstream; the origin not checked since the last commit")
    };
    Some(match (&activity.upstream, activity.ahead_behind) {
        (Some(upstream), Some((0, 0))) => {
            Mark::new(IN_SYNC, fetched(format!("Level with {upstream}")))
        }
        (Some(upstream), Some((ahead, 0))) => Mark::counted(
            AHEAD,
            &[ahead],
            fetched(format!("{ahead} ahead of {upstream}")),
        ),
        (Some(upstream), Some((0, behind))) => Mark::counted(
            BEHIND,
            &[behind],
            fetched(format!("{behind} behind {upstream}")),
        ),
        (Some(upstream), Some((ahead, behind))) => Mark::counted(
            DIVERGED,
            &[ahead, behind],
            fetched(format!("{ahead} ahead of and {behind} behind {upstream}")),
        ),
        (Some(upstream), None) => Mark::new(NO_BRANCH, format!("The upstream {upstream} is gone")),
        (None, _) => Mark::new(
            NO_UPSTREAM,
            format!("{} has no upstream", activity.branch_text()),
        ),
    })
}

/// The last commit's age, in the staleness colour, and whether the tree is dirty.
pub fn activity(row: &ProjectRow, now: DateTime<Utc>) -> Option<(Mark, Option<Mark>)> {
    let activity = row.badges.activity.as_ref()?;
    let Some(at) = activity.last_commit else {
        return Some((Mark::new(NO_COMMIT, "No commit yet"), None));
    };
    let sign = match activity.staleness(now) {
        Some(Staleness::Fresh) => FRESH,
        Some(Staleness::Quiet) | None => QUIET,
        Some(Staleness::Stale) => STALE,
    };
    let local = at.with_timezone(&chrono::Local).format("%Y-%m-%d %H:%M");
    let age = Mark::worded(
        sign,
        short_age(now - at),
        format!(
            "Last commit {local} ({}); {}",
            ago(now, at),
            activity.branch_text()
        ),
    );
    let dirty = (activity.changed > 0).then(|| {
        Mark::new(
            DIRTY,
            format!(
                "{} {} changed or untracked in the working tree",
                activity.changed,
                if activity.changed == 1 {
                    "path"
                } else {
                    "paths"
                }
            ),
        )
    });
    Some((age, dirty))
}

/// The latest GitHub Actions run, for a repository with workflows.
pub fn ci(row: &ProjectRow, now: DateTime<Utc>) -> Option<Mark> {
    if row.badges.workflows == 0 {
        return None;
    }
    let Some(fetched) = &row.badges.run else {
        return Some(Mark::new(
            RUN_UNASKED,
            "Workflows, and GitHub not asked yet",
        ));
    };
    let checked = format!("checked {}", ago(now, fetched.at));
    Some(match &fetched.value {
        Ok(Some(run)) => {
            let sign = match run.outcome() {
                Outcome::Passed => PASSED,
                Outcome::Failed => FAILED,
                Outcome::Running => RUNNING,
                Outcome::Other => OTHER_RUN,
            };
            let when = run
                .created_at
                .map(|at| format!(", started {}", ago(now, at)))
                .unwrap_or_default();
            let commit = run
                .head_sha
                .as_deref()
                .map(|sha| format!(" on {}", &sha[..sha.len().min(7)]))
                .unwrap_or_default();
            Mark::new(
                sign,
                format!("Latest run {}{commit}{when}; {checked}", run.text()),
            )
        }
        Ok(None) => Mark::new(OTHER_RUN, format!("Workflows, but no run yet; {checked}")),
        Err(reason) => Mark::new(
            RUN_UNREAD,
            format!("GitHub could not be asked ({checked}): {reason}"),
        ),
    })
}

/// A public repository without a license file (D-25's health).
pub fn license_warning(row: &ProjectRow) -> Option<Mark> {
    (row.badges.license.is_none() && row.badges.hosting.class() == Class::Public)
        .then(|| Mark::new(NO_LICENSE, "A public repository without a LICENSE file"))
}

/// The backup's age, from the Documents job (D-24).
pub fn backup(row: &ProjectRow, now: DateTime<Utc>) -> Option<Mark> {
    let backup = row.badges.backup.as_ref()?;
    let unit = backup.unit.as_deref().unwrap_or_default();
    let archive = backup
        .archive
        .as_ref()
        .map(|archive| format!("; the newest archive is {}", archive.name))
        .unwrap_or_default();
    let checked = backup.checked.map(|at| ago(now, at)).unwrap_or_default();
    Some(match backup.state(now) {
        BackupState::NotCovered => {
            Mark::new(NOT_COVERED, "No set of the backup job covers this folder")
        }
        BackupState::Unknown => Mark::new(
            BACKUP_UNKNOWN,
            format!("The backup job has no archive and no log line for {unit} yet"),
        ),
        BackupState::Fresh => Mark::new(
            BACKED_UP,
            format!("Backed up: the job wrote or checked {unit} {checked}{archive}"),
        ),
        BackupState::Stale => Mark::new(
            BACKUP_STALE,
            format!(
                "Stale: the job last wrote or checked {unit} {checked}, and it runs daily{archive}"
            ),
        ),
        BackupState::Failing => Mark::new(
            BACKUP_FAILING,
            format!(
                "The job's last word on {unit}: {}",
                backup.problem.as_deref().unwrap_or_default()
            ),
        ),
    })
}

/// The project's share of the week's usage (D-23, D-25; EP-18b): its sessions' weekly sum in
/// points of the limit, `+0 %` weak when they worked through no rise. The sentence adds Fable's
/// sum and the project's part of all the points the window attributed this week.
pub fn usage(row: &ProjectRow) -> Option<Mark> {
    let usage = row.badges.usage?;
    let weekly = usage.weekly?;
    let points = ledger::points(weekly);
    let sign = if points == "0" { UNMOVED } else { USED };
    let fable = usage
        .fable
        .map(|fable| format!(" and {} of Fable's", ledger::points(fable)))
        .unwrap_or_default();
    let sessions = match usage.sessions {
        1 => "1 session".to_owned(),
        n => format!("{n} sessions"),
    };
    let since = usage.since.with_timezone(&Local).format("%a %d %b %H:%M");
    let part = match ledger::points(usage.attributed).as_str() {
        "0" => String::new(),
        all => format!(
            ": {:.0} % of the {} the window attributed",
            weekly / usage.attributed * 100.0,
            of_points(all)
        ),
    };
    Some(Mark::worded(
        sign,
        format!("+{points} %"),
        format!(
            "{} of the weekly limit{fable} over {sessions} this week, since the reset on \
             {since}{part}",
            of_points(&points)
        ),
    ))
}

/// `1 point`, `2.5 points`.
fn of_points(points: &str) -> String {
    match points {
        "1" => "1 point".to_owned(),
        points => format!("{points} points"),
    }
}

/// Space between the legend's groups, side by side.
const GROUP_GAP: f32 = 28.0;

/// The key to the row's marks (EP-7a): the stack chips, then each group of [`KEY`], in as many
/// columns as the width holds, each column as wide as its widest group; in a group, each glyph in
/// a column of the group's widest glyph, then its meaning.
pub fn legend(ui: &mut egui::Ui) {
    let groups = KEY.len() + 1;
    let widths: Vec<f32> = (0..groups)
        .map(|group| {
            style::width_of(ui, |ui| {
                ui.vertical(|ui| key_group(ui, group));
            })
        })
        .collect();
    let room = ui.available_width();
    let columns = (1..=groups)
        .rev()
        .map(|per_line| {
            (0..per_line)
                .map(|column| {
                    widths
                        .iter()
                        .skip(column)
                        .step_by(per_line)
                        .copied()
                        .fold(0.0, f32::max)
                })
                .collect::<Vec<f32>>()
        })
        .find(|columns| {
            columns.iter().sum::<f32>() + GROUP_GAP * (columns.len() - 1) as f32 <= room
        })
        .unwrap_or_else(|| vec![widths.iter().copied().fold(0.0, f32::max)]);
    let order: Vec<usize> = (0..groups).collect();
    for line in order.chunks(columns.len()) {
        ui.horizontal_top(|ui| {
            for (&group, &width) in line.iter().zip(&columns) {
                ui.allocate_ui_with_layout(vec2(width, 0.0), Layout::top_down(Align::Min), |ui| {
                    ui.set_min_width(width);
                    key_group(ui, group);
                });
                ui.add_space(GROUP_GAP - ui.spacing().item_spacing.x);
            }
        });
        ui.add_space(4.0);
    }
}

/// The legend's group `index`: 0 the stack chips, then [`KEY`]'s groups.
fn key_group(ui: &mut egui::Ui, index: usize) {
    // Lines as tall as their text, not a button's height.
    ui.spacing_mut().item_spacing.y = 1.0;
    ui.spacing_mut().interact_size.y = ui.text_style_height(&egui::TextStyle::Body);
    let group = index.checked_sub(1).and_then(|i| KEY.get(i));
    ui.label(
        RichText::new(group.map_or("Stack", |group| group.title))
            .small()
            .strong(),
    );
    let meaning = |ui: &mut egui::Ui, text: &str| {
        ui.label(RichText::new(text).small());
    };
    match group {
        None => {
            let chips: Vec<(String, Color32, &str)> = Kind::ALL
                .into_iter()
                .map(|kind| {
                    (
                        kind.short().to_owned(),
                        style::stack_colour(kind),
                        kind.label(),
                    )
                })
                .chain([(
                    format!("{}+1", Kind::Rust.short()),
                    style::stack_colour(Kind::Rust),
                    "and n more stacks",
                )])
                .collect();
            let column = chips
                .iter()
                .map(|(text, colour, _)| {
                    style::width_of(ui, |ui| {
                        style::chip(ui, text, *colour);
                    })
                })
                .fold(0.0, f32::max);
            for (text, colour, label) in &chips {
                ui.horizontal(|ui| {
                    style::padded(ui, column, |ui| {
                        style::chip(ui, text, *colour);
                    });
                    meaning(ui, label);
                });
            }
        }
        Some(group) => {
            let column = group
                .signs
                .iter()
                .map(|sign| {
                    style::width_of(ui, |ui| {
                        ui.label(sign.rich(ui));
                    })
                })
                .fold(0.0, f32::max);
            for sign in group.signs {
                ui.horizontal(|ui| {
                    style::padded(ui, column, |ui| {
                        ui.label(sign.rich(ui));
                    });
                    meaning(ui, sign.meaning);
                });
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    use chrono::TimeDelta;
    use crw_core::badges::activity::Activity;
    use crw_core::badges::backup::Backup;
    use crw_core::badges::health::Run;
    use crw_core::badges::hosting::{Check, Listed, Repo};
    use crw_core::discovery::Folder;
    use crw_core::proposal::Facts;
    use crw_core::registry::{Project, State};
    use crw_core::roadmap::snapshot::Snapshot;

    use super::*;
    use crate::portfolio::{Badges, Fetched, Usage};

    fn row(badges: Badges) -> ProjectRow {
        ProjectRow {
            project: Project::new("alpha", Path::new("alpha"), State::Active),
            path: PathBuf::from("alpha"),
            folder: Some(Folder {
                name: "alpha".to_owned(),
                path: PathBuf::from("alpha"),
                git: true,
                roadmap: true,
                own_workspace: true,
            }),
            snapshot: Snapshot::default(),
            facts: Facts::default(),
            outgoing: Vec::new(),
            mail: Vec::new(),
            git: None,
            badges,
            house: Default::default(),
            history: None,
            reread: None,
        }
    }

    fn group(title: &str) -> &'static [Sign] {
        KEY.iter()
            .find(|group| group.title == title)
            .map(|group| group.signs)
            .unwrap()
    }

    fn repo() -> Repo {
        Repo {
            owner: "example".to_owned(),
            name: "alpha".to_owned(),
        }
    }

    fn listed(visibility: Visibility, archived: bool) -> Listed {
        Listed {
            repo: repo(),
            visibility,
            archived,
            pushed_at: None,
            default_branch: Some("main".to_owned()),
        }
    }

    fn activity(last_commit: Option<DateTime<Utc>>, upstream: Option<(u32, u32)>) -> Activity {
        Activity {
            oid: None,
            last_commit,
            changed: 2,
            branch: Some("main".to_owned()),
            upstream: upstream.map(|_| "origin/main".to_owned()),
            ahead_behind: upstream,
        }
    }

    #[test]
    fn every_mark_is_made_from_a_sign_of_its_legend_group() {
        let now = Utc::now();
        let hostings = [
            Hosting::Listed(listed(Visibility::Public, false)),
            Hosting::Listed(listed(Visibility::Private, false)),
            Hosting::Listed(listed(Visibility::Internal, false)),
            Hosting::Listed(listed(Visibility::Public, true)),
            Hosting::Local,
            Hosting::Orphan(listed(Visibility::Private, false)),
            Hosting::Missing(repo()),
            Hosting::GitHub(repo()),
            Hosting::Elsewhere("example.org".to_owned()),
        ];
        for hosting in hostings {
            let mark = super::hosting(
                &row(Badges {
                    hosting,
                    ..Badges::default()
                }),
                now,
            );
            assert!(group("Hosting").contains(&mark.sign), "{mark:?}");
        }

        let remote = Some("https://github.com/example/alpha.git".to_owned());
        let verdicts = [
            Verdict::InSync,
            Verdict::Ahead(2),
            Verdict::Behind(3),
            Verdict::Diverged {
                ahead: 1,
                behind: 4,
            },
            Verdict::NotFetched,
            Verdict::NoBranch,
            Verdict::Failed("no network".to_owned()),
        ];
        let checked = verdicts.into_iter().map(|verdict| Badges {
            remote: remote.clone(),
            head: Some(Fetched {
                value: Check {
                    remote_ref: "refs/heads/main".to_owned(),
                    remote: None,
                    local: None,
                    verdict,
                },
                at: now,
            }),
            ..Badges::default()
        });
        let counted = [Some((0, 0)), Some((2, 0)), Some((0, 3)), Some((1, 4)), None]
            .into_iter()
            .map(|counts| Badges {
                remote: remote.clone(),
                activity: Some(activity(Some(now), counts)),
                ..Badges::default()
            });
        let gone = Badges {
            remote: remote.clone(),
            activity: Some(Activity {
                ahead_behind: None,
                ..activity(Some(now), Some((0, 0)))
            }),
            ..Badges::default()
        };
        for badges in checked.chain(counted).chain([gone]) {
            let mark = sync(&row(badges), now).unwrap();
            assert!(group("Sync").contains(&mark.sign), "{mark:?}");
        }

        for last in [
            Some(now - TimeDelta::days(2)),
            Some(now - TimeDelta::days(20)),
            Some(now - TimeDelta::days(90)),
            None,
        ] {
            let row = row(Badges {
                activity: Some(activity(last, None)),
                ..Badges::default()
            });
            let (age, dirty) = super::activity(&row, now).unwrap();
            assert!(group("Age").contains(&age.sign), "{age:?}");
            if let Some(dirty) = dirty {
                assert!(group("Age").contains(&dirty.sign), "{dirty:?}");
            }
        }

        let run = |status: &str, conclusion: Option<&str>| {
            Ok(Some(Run {
                workflow: "ci".to_owned(),
                status: status.to_owned(),
                conclusion: conclusion.map(str::to_owned),
                created_at: None,
                head_sha: None,
            }))
        };
        let runs = [
            None,
            Some(run("completed", Some("success"))),
            Some(run("completed", Some("failure"))),
            Some(run("in_progress", None)),
            Some(run("completed", Some("cancelled"))),
            Some(Ok(None)),
            Some(Err("no network".to_owned())),
        ];
        for run in runs {
            let row = row(Badges {
                workflows: 1,
                run: run.map(|value| Fetched { value, at: now }),
                ..Badges::default()
            });
            let mark = ci(&row, now).unwrap();
            assert!(group("CI and license").contains(&mark.sign), "{mark:?}");
        }
        let unlicensed = row(Badges {
            hosting: Hosting::Listed(listed(Visibility::Public, false)),
            ..Badges::default()
        });
        let warning = license_warning(&unlicensed).unwrap();
        assert!(group("CI and license").contains(&warning.sign));

        let unit = Some("WORK\\alpha".to_owned());
        let backups = [
            Backup::default(),
            Backup {
                unit: unit.clone(),
                problem: Some("in use".to_owned()),
                ..Backup::default()
            },
            Backup {
                unit: unit.clone(),
                ..Backup::default()
            },
            Backup {
                unit: unit.clone(),
                checked: Some(now - TimeDelta::hours(5)),
                ..Backup::default()
            },
            Backup {
                unit,
                checked: Some(now - TimeDelta::hours(70)),
                ..Backup::default()
            },
        ];
        let mut seen = Vec::new();
        for backup in backups {
            let row = row(Badges {
                backup: Some(backup),
                ..Badges::default()
            });
            let mark = super::backup(&row, now).unwrap();
            assert!(group("Backup").contains(&mark.sign), "{mark:?}");
            seen.push(mark.sign);
        }
        // Each of the job's five states has a sign of its own.
        assert_eq!(seen.len(), group("Backup").len());
        assert!(group("Backup").iter().all(|sign| seen.contains(sign)));

        let since = now - TimeDelta::days(2);
        let usage = |weekly, fable| Badges {
            usage: Some(Usage {
                weekly,
                fable,
                sessions: 2,
                since,
                attributed: 12.0,
            }),
            ..Badges::default()
        };
        let mut shown = Vec::new();
        for (weekly, fable) in [(6.0, Some(4.5)), (0.04, None), (0.0, Some(0.0))] {
            let mark = super::usage(&row(usage(Some(weekly), fable))).unwrap();
            assert!(group("Usage").contains(&mark.sign), "{mark:?}");
            shown.push(mark.sign);
        }
        // Under a twentieth of a point writes `0`, weak like none at all.
        assert_eq!(shown, [USED, UNMOVED, UNMOVED]);
        // No mark without a weekly share: Fable's alone, or none.
        assert!(super::usage(&row(usage(None, Some(1.0)))).is_none());
        assert!(super::usage(&row(Badges::default())).is_none());
    }

    #[test]
    fn a_usage_mark_gives_the_weeks_points_and_the_projects_part() {
        let since = DateTime::parse_from_rfc3339("2026-10-31T22:00:00Z")
            .unwrap()
            .with_timezone(&Utc);
        let local = since.with_timezone(&Local).format("%a %d %b %H:%M");
        let mark = |weekly, fable, sessions, attributed| {
            usage(&row(Badges {
                usage: Some(Usage {
                    weekly: Some(weekly),
                    fable,
                    sessions,
                    since,
                    attributed,
                }),
                ..Badges::default()
            }))
            .unwrap()
        };
        let six = mark(6.0, Some(4.5), 3, 12.0);
        assert_eq!((six.sign, six.text.as_deref()), (USED, Some("+6 %")));
        assert_eq!(
            six.hover,
            format!(
                "6 points of the weekly limit and 4.5 of Fable's over 3 sessions this week, \
                 since the reset on {local}: 50 % of the 12 points the window attributed"
            )
        );
        let third = mark(1.0 / 3.0, None, 1, 1.0);
        assert_eq!(third.text.as_deref(), Some("+0.3 %"));
        assert_eq!(
            third.hover,
            format!(
                "0.3 points of the weekly limit over 1 session this week, since the reset on \
                 {local}: 33 % of the 1 point the window attributed"
            )
        );
        // Nothing attributed yet: no part to give.
        let none = mark(0.0, Some(0.0), 1, 0.0);
        assert_eq!((none.sign, none.text.as_deref()), (UNMOVED, Some("+0 %")));
        assert_eq!(
            none.hover,
            format!(
                "0 points of the weekly limit and 0 of Fable's over 1 session this week, since \
                 the reset on {local}"
            )
        );
    }

    #[test]
    fn no_two_signs_of_a_group_look_alike() {
        for group in &KEY {
            for (i, a) in group.signs.iter().enumerate() {
                for b in &group.signs[i + 1..] {
                    assert!(
                        a.glyph != b.glyph || a.tone != b.tone,
                        "{}: {a:?} and {b:?}",
                        group.title
                    );
                }
            }
        }
    }

    #[test]
    fn a_counted_mark_writes_its_numbers_into_the_signs_sample() {
        assert_eq!(
            Mark::counted(DIVERGED, &[2, 5], "").text.as_deref(),
            Some("↑2↓5")
        );
        assert_eq!(Mark::counted(AHEAD, &[3], "").text.as_deref(), Some("↑3"));
        // An icon has no sample to write into.
        assert_eq!(Mark::counted(NOT_FETCHED, &[1], "").text, None);
        assert_ne!(Glyph::Text("?"), Glyph::Icon(icons::ICON_HELP));
    }
}
