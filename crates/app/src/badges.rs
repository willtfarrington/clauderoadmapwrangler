//! The badges as the window draws them (D-24, D-25): each reading becomes a [`Mark`], an icon or a
//! few characters in a tone with the whole sentence as hover text, for the row's badge column and
//! the Detail pane's Status section.

use chrono::{DateTime, Utc};
use crw_core::badges::activity::Staleness;
use crw_core::badges::backup::State as BackupState;
use crw_core::badges::health::Outcome;
use crw_core::badges::hosting::{Class, Hosting, Verdict, Visibility};
use crw_core::badges::short_age;
use eframe::egui::{self, Color32, RichText};
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

/// One badge: an icon or a short text, its tone, and the sentence behind it.
#[derive(Clone, Debug)]
pub struct Mark {
    pub icon: Option<MaterialIcon>,
    pub text: Option<String>,
    pub tone: Tone,
    pub hover: String,
}

impl Mark {
    fn icon(icon: MaterialIcon, tone: Tone, hover: impl Into<String>) -> Self {
        Self {
            icon: Some(icon),
            text: None,
            tone,
            hover: hover.into(),
        }
    }

    fn text(text: impl Into<String>, tone: Tone, hover: impl Into<String>) -> Self {
        Self {
            icon: None,
            text: Some(text.into()),
            tone,
            hover: hover.into(),
        }
    }

    /// Draws the mark where the layout puts it, with its hover text.
    pub fn show(&self, ui: &mut egui::Ui) -> egui::Response {
        let colour = self.tone.colour(ui);
        let text = match (self.icon, &self.text) {
            (Some(icon), _) => style::icon(icon, Some(colour)),
            (None, Some(text)) => RichText::new(text).color(colour),
            (None, None) => RichText::new(""),
        };
        ui.label(text).on_hover_text(&self.hover)
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
            let (icon, tone, word) = match (listed.archived, listed.visibility) {
                (true, _) => (icons::ICON_INVENTORY_2, Tone::Weak, "Archived"),
                (false, Visibility::Public) => (icons::ICON_PUBLIC, Tone::Good, "Public"),
                (false, Visibility::Private) => (icons::ICON_LOCK, Tone::Plain, "Private"),
                (false, Visibility::Internal) => (icons::ICON_LOCK, Tone::Plain, "Internal"),
            };
            let mut hover = format!("{word} on GitHub: {}", listed.repo.slug());
            if let Some(at) = listed.pushed_at {
                hover.push_str(&format!(", pushed {}", ago(now, at)));
            }
            if let Some(branch) = &listed.default_branch {
                hover.push_str(&format!(", default branch {branch}"));
            }
            Mark::icon(icon, tone, hover)
        }
        Hosting::Local => Mark::icon(icons::ICON_LAPTOP, Tone::Weak, "Local only: no remote"),
        Hosting::Orphan(listed) => Mark::icon(
            icons::ICON_LINK_OFF,
            Tone::Warn,
            format!(
                "Orphan: no remote here, but the account holds {}{}",
                listed.repo.slug(),
                listed
                    .pushed_at
                    .map(|at| format!(", last pushed {}", ago(now, at)))
                    .unwrap_or_default()
            ),
        ),
        Hosting::Missing(repo) => Mark::icon(
            icons::ICON_CLOUD_OFF,
            Tone::Bad,
            format!(
                "Missing: the origin names {}, which the account does not list",
                repo.slug()
            ),
        ),
        Hosting::GitHub(repo) => Mark::icon(
            icons::ICON_CLOUD,
            Tone::Weak,
            format!(
                "On GitHub: {} (not in the account's list, or the list not read yet)",
                repo.slug()
            ),
        ),
        Hosting::Elsewhere(host) => {
            Mark::icon(icons::ICON_DNS, Tone::Plain, format!("A remote at {host}"))
        }
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
            Verdict::InSync => Mark::text(
                "=",
                Tone::Weak,
                format!("In sync: {checked} ({})", check.remote_ref),
            ),
            Verdict::Ahead(n) => Mark::text(
                format!("↑{n}"),
                Tone::Warn,
                format!("{} not pushed: {checked}", commits(*n)),
            ),
            Verdict::Behind(n) => Mark::text(
                format!("↓{n}"),
                Tone::Warn,
                format!("{} on the origin not pulled: {checked}", commits(*n)),
            ),
            Verdict::Diverged { ahead, behind } => Mark::text(
                format!("↑{ahead}↓{behind}"),
                Tone::Bad,
                format!("Diverged, {ahead} ahead and {behind} behind: {checked}"),
            ),
            Verdict::NotFetched => Mark::icon(
                icons::ICON_SYNC_PROBLEM,
                Tone::Warn,
                format!(
                    "The origin holds commits this clone has never fetched, so it is behind or \
                     diverged: {checked}"
                ),
            ),
            Verdict::NoBranch => Mark::text(
                "?",
                Tone::Weak,
                format!("The origin has no {}: {checked}", check.remote_ref),
            ),
            Verdict::Failed(reason) => Mark::icon(
                icons::ICON_SYNC_PROBLEM,
                Tone::Weak,
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
            Mark::text("=", Tone::Weak, fetched(format!("Level with {upstream}")))
        }
        (Some(upstream), Some((ahead, 0))) => Mark::text(
            format!("↑{ahead}"),
            Tone::Warn,
            fetched(format!("{ahead} ahead of {upstream}")),
        ),
        (Some(upstream), Some((0, behind))) => Mark::text(
            format!("↓{behind}"),
            Tone::Warn,
            fetched(format!("{behind} behind {upstream}")),
        ),
        (Some(upstream), Some((ahead, behind))) => Mark::text(
            format!("↑{ahead}↓{behind}"),
            Tone::Bad,
            fetched(format!("{ahead} ahead of and {behind} behind {upstream}")),
        ),
        (Some(upstream), None) => {
            Mark::text("?", Tone::Weak, format!("The upstream {upstream} is gone"))
        }
        (None, _) => Mark::text(
            "–",
            Tone::Weak,
            format!("{} has no upstream", activity.branch_text()),
        ),
    })
}

/// The last commit's age, in the staleness colour, and whether the tree is dirty.
pub fn activity(row: &ProjectRow, now: DateTime<Utc>) -> Option<(Mark, Option<Mark>)> {
    let activity = row.badges.activity.as_ref()?;
    let Some(at) = activity.last_commit else {
        return Some((Mark::text("new", Tone::Weak, "No commit yet"), None));
    };
    let tone = match activity.staleness(now) {
        Some(Staleness::Fresh) => Tone::Plain,
        Some(Staleness::Quiet) | None => Tone::Weak,
        Some(Staleness::Stale) => Tone::Warn,
    };
    let local = at.with_timezone(&chrono::Local).format("%Y-%m-%d %H:%M");
    let age = Mark::text(
        short_age(now - at),
        tone,
        format!(
            "Last commit {local} ({}); {}",
            ago(now, at),
            activity.branch_text()
        ),
    );
    let dirty = (activity.changed > 0).then(|| {
        Mark::text(
            "●",
            Tone::Warn,
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
        return Some(Mark::icon(
            icons::ICON_HOURGLASS_EMPTY,
            Tone::Weak,
            "Workflows, and GitHub not asked yet",
        ));
    };
    let checked = format!("checked {}", ago(now, fetched.at));
    Some(match &fetched.value {
        Ok(Some(run)) => {
            let (icon, tone) = match run.outcome() {
                Outcome::Passed => (icons::ICON_CHECK_CIRCLE, Tone::Good),
                Outcome::Failed => (icons::ICON_CANCEL, Tone::Bad),
                Outcome::Running => (icons::ICON_PENDING, Tone::Warn),
                Outcome::Other => (icons::ICON_HELP, Tone::Weak),
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
            Mark::icon(
                icon,
                tone,
                format!("Latest run {}{commit}{when}; {checked}", run.text()),
            )
        }
        Ok(None) => Mark::icon(
            icons::ICON_HELP,
            Tone::Weak,
            format!("Workflows, but no run yet; {checked}"),
        ),
        Err(reason) => Mark::icon(
            icons::ICON_SYNC_PROBLEM,
            Tone::Weak,
            format!("GitHub could not be asked ({checked}): {reason}"),
        ),
    })
}

/// A public repository without a licence file (D-25's health).
pub fn licence_warning(row: &ProjectRow) -> Option<Mark> {
    (row.badges.licence.is_none() && row.badges.hosting.class() == Class::Public).then(|| {
        Mark::icon(
            icons::ICON_GAVEL,
            Tone::Warn,
            "A public repository without a LICENSE file",
        )
    })
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
        BackupState::NotCovered => Mark::icon(
            icons::ICON_CLOUD_OFF,
            Tone::Warn,
            "No set of the backup job covers this folder",
        ),
        BackupState::Unknown => Mark::icon(
            icons::ICON_HELP,
            Tone::Weak,
            format!("The backup job has no archive and no log line for {unit} yet"),
        ),
        BackupState::Fresh => Mark::icon(
            icons::ICON_BACKUP,
            Tone::Good,
            format!("Backed up: the job wrote or checked {unit} {checked}{archive}"),
        ),
        BackupState::Stale => Mark::icon(
            icons::ICON_BACKUP,
            Tone::Warn,
            format!(
                "Stale: the job last wrote or checked {unit} {checked}, and it runs daily{archive}"
            ),
        ),
        BackupState::Failing => Mark::icon(
            icons::ICON_ERROR,
            Tone::Bad,
            format!(
                "The job's last word on {unit}: {}",
                backup.problem.as_deref().unwrap_or_default()
            ),
        ),
    })
}
