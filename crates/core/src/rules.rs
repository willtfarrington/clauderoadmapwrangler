//! The rule table (D-22): the model, effort and ultracode a brief launches with, resolved in layers,
//! each value with the reason it was chosen, which the launch dialog shows beside it: the tool's
//! default (`fable` at `xhigh`), the project's launch defaults, the first global rule that matches
//! the brief, then the first of the project's own rules. A rule matches by size, by phase and by
//! keywords of the title (every condition it gives must hold), and sets a model, an effort or a
//! step along the effort levels, and ultracode. While the registry lists no global rule, the
//! built-in table stands in for it: an S brief one effort step lower. The permission mode is the
//! project's, else `acceptEdits` (D-18), and so is the launch mode, else the tool's (EP-11).
//! Last comes the downshift (D-23; EP-18): an S brief that would run on Fable is proposed
//! [`DOWNSHIFT_MODEL`] at the same effort while the meter's Fable figure is above the registry's
//! mark. Nothing here writes the registry: the owner writes rules by hand (rule 13).

use std::fmt;

use chrono::{DateTime, Utc};
use serde::Deserialize;

use crate::badges::short_age;
use crate::registry::{LaunchMode, Project, Registry};
use crate::roadmap::Size;
use crate::usage::{self, Figures, Limit, Reading};

/// The model a brief launches with when nothing else is set (D-22): an alias, passed as written.
pub const DEFAULT_MODEL: &str = "fable";
pub const DEFAULT_EFFORT: &str = "xhigh";
/// Edits in the project pass, everything else reaches the host (D-18).
pub const DEFAULT_PERMISSION_MODE: &str = "acceptEdits";
/// The permission modes the tool knows (D-18; EP-19b), from Claude Code's permission-modes page:
/// `manual` is its Manual mode, which the CLI also names `default`, a word `crw set` keeps for
/// taking the project's mode out.
pub const PERMISSION_MODES: [&str; 6] = [
    "acceptEdits",
    "auto",
    "plan",
    "dontAsk",
    "bypassPermissions",
    "manual",
];
/// Visual (D-16's EP-16 addendum): with no allow rule, a headless run under `acceptEdits` waits
/// for the owner at every shell command; this project's briefs end with a release build that
/// needs the window quit, which ends the window's headless runs; and a brief's closing rounds of
/// questions and trials suit a VS Code tab. The autopilot runs headless whatever it is (EP-19).
pub const DEFAULT_LAUNCH_MODE: LaunchMode = LaunchMode::Visual;

/// The model the downshift proposes for an S brief while Fable's weekly share is high (D-23).
pub const DOWNSHIFT_MODEL: &str = "claude-opus-5-5";
/// The Fable share above which it does, unless the registry sets another mark (D-23).
pub const DEFAULT_DOWNSHIFT_PERCENT: u8 = 60;

/// The effort levels in order, for a step, when neither the CLI nor the catalog lists them.
pub const EFFORTS: [&str; 5] = ["low", "medium", "high", "xhigh", "max"];

/// A `[[rule]]` of the registry, or a `[[project.rule]]` of one project.
#[derive(Debug, Clone, Default, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Rule {
    /// Shown as the reason; without it, the rule's match in words.
    pub name: Option<String>,
    #[serde(rename = "match", default)]
    pub when: Match,
    #[serde(default)]
    pub set: Set,
}

/// What a rule matches. Every condition it gives must hold; one that gives none matches every
/// brief.
#[derive(Debug, Clone, Default, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Match {
    /// `S`, `M` or `L`, one or several.
    #[serde(default)]
    pub size: Words,
    /// Phase ids as the phase headings write them: `P2`.
    #[serde(default)]
    pub phase: Words,
    /// Words or phrases, any of which the row's title holds, case aside: `re-plan`.
    #[serde(default)]
    pub title: Words,
}

/// What a rule sets; a field it leaves out keeps the value of the layers below.
#[derive(Debug, Clone, Default, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Set {
    pub model: Option<String>,
    pub effort: Option<String>,
    /// Steps along the effort levels from the effort resolved so far: `-1` is one lower.
    pub effort_step: Option<i64>,
    pub ultracode: Option<bool>,
}

/// One word or a list of them, as a hand-written registry may give either.
#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
#[serde(from = "OneOrMany")]
pub struct Words(pub Vec<String>);

#[derive(Deserialize)]
#[serde(untagged)]
enum OneOrMany {
    One(String),
    Many(Vec<String>),
}

impl From<OneOrMany> for Words {
    fn from(value: OneOrMany) -> Self {
        match value {
            OneOrMany::One(word) => Words(vec![word]),
            OneOrMany::Many(words) => Words(words),
        }
    }
}

impl Words {
    fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    fn joined(&self, or: &str) -> String {
        self.0
            .iter()
            .map(|word| word.trim())
            .collect::<Vec<_>>()
            .join(or)
    }
}

/// What the rules look at in a brief: its row's size, its phase's id and its title.
#[derive(Debug, Clone, Copy, Default)]
pub struct Brief<'a> {
    pub size: Option<Size>,
    pub phase: Option<&'a str>,
    pub title: &'a str,
}

impl Match {
    pub fn matches(&self, brief: &Brief<'_>) -> bool {
        let size = self.size.is_empty()
            || brief.size.is_some_and(|size| {
                self.size
                    .0
                    .iter()
                    .any(|s| s.trim().eq_ignore_ascii_case(size.as_str()))
            });
        let phase = self.phase.is_empty()
            || brief.phase.is_some_and(|phase| {
                self.phase
                    .0
                    .iter()
                    .any(|p| p.trim().eq_ignore_ascii_case(phase))
            });
        let title = self.title.is_empty() || {
            let title = brief.title.to_lowercase();
            self.title
                .0
                .iter()
                .any(|word| title.contains(&word.trim().to_lowercase()))
        };
        size && phase && title
    }

    /// The match in words: `size S`, `size S or M, title "re-plan"`, `every brief`.
    pub fn describe(&self) -> String {
        let mut parts = Vec::new();
        if !self.size.is_empty() {
            parts.push(format!("size {}", self.size.joined(" or ")));
        }
        if !self.phase.is_empty() {
            parts.push(format!("phase {}", self.phase.joined(" or ")));
        }
        if !self.title.is_empty() {
            let words: Vec<String> = self
                .title
                .0
                .iter()
                .map(|w| format!("{:?}", w.trim()))
                .collect();
            parts.push(format!("title {}", words.join(" or ")));
        }
        if parts.is_empty() {
            "every brief".to_owned()
        } else {
            parts.join(", ")
        }
    }
}

impl Rule {
    /// The rule in words: its name, else its match.
    pub fn label(&self) -> String {
        match &self.name {
            Some(name) if !name.trim().is_empty() => name.trim().to_owned(),
            _ => self.when.describe(),
        }
    }

    /// The checks serde cannot make, in words.
    pub(crate) fn check(&self) -> Result<(), String> {
        for size in &self.when.size.0 {
            if !matches!(size.trim().to_ascii_uppercase().as_str(), "S" | "M" | "L") {
                return Err(format!("its size is S, M or L, not {size:?}"));
            }
        }
        for (key, words) in [("phase", &self.when.phase), ("title", &self.when.title)] {
            if words.0.iter().any(|word| word.trim().is_empty()) {
                return Err(format!("its {key} names an empty word"));
            }
        }
        let set = &self.set;
        if set.model.is_none()
            && set.effort.is_none()
            && set.effort_step.is_none()
            && set.ultracode.is_none()
        {
            return Err("it sets nothing: model, effort, effort_step or ultracode".to_owned());
        }
        if set.effort.is_some() && set.effort_step.is_some() {
            return Err("it sets effort or effort_step, not both".to_owned());
        }
        if set.effort_step == Some(0) {
            return Err("its effort_step 0 changes nothing".to_owned());
        }
        for (key, value) in [("model", &set.model), ("effort", &set.effort)] {
            if value.as_ref().is_some_and(|value| value.trim().is_empty()) {
                return Err(format!("its {key} is empty"));
            }
        }
        Ok(())
    }
}

/// The built-in table (D-22), standing in while the registry lists no global rule.
pub fn built_in() -> Vec<Rule> {
    vec![Rule {
        name: Some("an S brief runs one effort step lower (D-22)".to_owned()),
        when: Match {
            size: Words(vec!["S".to_owned()]),
            ..Match::default()
        },
        set: Set {
            effort_step: Some(-1),
            ..Set::default()
        },
    }]
}

/// Where a launch's value came from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Source {
    /// The tool's own default (D-18, D-22).
    Default,
    /// The project's `[project.launch]`.
    Project,
    /// The built-in table's rule, by its label.
    BuiltIn(String),
    /// The registry's `[[rule]]` number n (from 1), by its label.
    Rule(usize, String),
    /// The project's own `[[project.rule]]` number n, by its label.
    ProjectRule(usize, String),
    /// The Fable downshift (D-23), its figure in the choice's detail.
    Downshift,
    /// The owner's choice in the launch dialog.
    Owner,
}

impl fmt::Display for Source {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Source::Default => f.write_str("the tool's default"),
            Source::Project => f.write_str("the project's launch defaults"),
            Source::BuiltIn(label) => write!(f, "the built-in rule: {label}"),
            Source::Rule(n, label) => write!(f, "rule {n}: {label}"),
            Source::ProjectRule(n, label) => write!(f, "the project's rule {n}: {label}"),
            Source::Downshift => f.write_str("the Fable downshift (D-23)"),
            Source::Owner => f.write_str("chosen here"),
        }
    }
}

/// The Fable figure the downshift weighs: the Fable-scoped weekly limit as the endpoint last gave
/// it, and when.
#[derive(Debug, Clone, PartialEq)]
pub struct Fable {
    pub limit: Limit,
    pub at: DateTime<Utc>,
}

/// What the downshift weighs (D-23; EP-18): the meter's Fable figure, none before its first
/// reading, and the time to judge its age at.
#[derive(Debug, Clone, PartialEq)]
pub struct Usage {
    pub fable: Option<Fable>,
    pub now: DateTime<Utc>,
}

impl Usage {
    /// Nothing read: the downshift says it cannot judge.
    pub fn unread(now: DateTime<Utc>) -> Self {
        Self { fable: None, now }
    }

    /// The window's meter (`usage::Gauge::figures`).
    pub fn of(figures: &Figures, now: DateTime<Utc>) -> Self {
        Self {
            fable: figures.fable.as_ref().map(|figure| Fable {
                limit: figure.limit.clone(),
                at: figure.at,
            }),
            now,
        }
    }

    /// An endpoint's reading taken at `at`: the ledger's cached one, for `crw prompt`.
    pub fn of_reading(reading: &Reading, at: DateTime<Utc>, now: DateTime<Utc>) -> Self {
        Self {
            fable: reading.fable().map(|limit| Fable {
                limit: limit.clone(),
                at,
            }),
            now,
        }
    }
}

/// How the downshift judged a brief it weighs: an S brief whose model so far is a Fable model.
#[derive(Debug, Clone, PartialEq)]
pub struct Downshift {
    /// Whether it proposed [`DOWNSHIFT_MODEL`].
    pub applied: bool,
    /// The figure against the mark, in words: `Fable at 64 %, read 3 min ago, above 60 %`.
    pub words: String,
}

/// Whether `model` names a Fable model: the alias or one of its ids.
pub fn is_fable(model: &str) -> bool {
    model.to_ascii_lowercase().contains("fable")
}

/// A value and why it was chosen.
#[derive(Debug, Clone, PartialEq)]
pub struct Choice<T> {
    pub value: T,
    pub source: Source,
    /// What the source did, when it did more than set the value: `one step lower than xhigh`.
    pub detail: Option<String>,
}

impl<T> Choice<T> {
    fn new(value: T, source: Source) -> Self {
        Self {
            value,
            source,
            detail: None,
        }
    }

    /// The reason in words, for the dialog and `crw prompt`.
    pub fn reason(&self) -> String {
        match &self.detail {
            Some(detail) => format!("{} ({detail})", self.source),
            None => self.source.to_string(),
        }
    }
}

/// A model, effort and ultracode as the owner set them (EP-19a): the chain plan's choice for a
/// brief, which the chain takes in place of the rules'.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Picked {
    pub model: String,
    /// `None` leaves the effort to the CLI.
    pub effort: Option<String>,
    pub ultracode: bool,
}

impl Picked {
    /// The resolution's values, as the pickers show them before the owner touches one.
    pub fn of(resolution: &Resolution) -> Self {
        Self {
            model: resolution.model.value.clone(),
            effort: resolution.effort.value.clone(),
            ultracode: resolution.ultracode.value,
        }
    }
}

/// A brief's launch values, each with its reason.
#[derive(Debug, Clone, PartialEq)]
pub struct Resolution {
    /// Visual or headless (D-16).
    pub mode: Choice<LaunchMode>,
    pub model: Choice<String>,
    /// `None` leaves the effort to the CLI.
    pub effort: Choice<Option<String>>,
    pub ultracode: Choice<bool>,
    pub permission_mode: Choice<String>,
    /// What a rule asked for and could not be done, in words.
    pub notes: Vec<String>,
    /// The downshift's judgement, for the briefs it weighs (D-23).
    pub downshift: Option<Downshift>,
}

/// The permission mode a project's sessions start in (D-18; EP-19b): its `[project.launch]
/// permission_mode`, else the tool's [`DEFAULT_PERMISSION_MODE`]. No rule sets one.
pub fn permission_mode(project: &Project) -> Choice<String> {
    match &project.launch.permission_mode {
        Some(mode) => Choice::new(mode.trim().to_owned(), Source::Project),
        None => Choice::new(DEFAULT_PERMISSION_MODE.to_owned(), Source::Default),
    }
}

/// Resolves `brief`'s launch values for `project`; `levels` are the effort levels in order that a
/// step moves along (the built-in [`EFFORTS`] when empty), and `usage` is what the downshift
/// weighs, last of the layers.
pub fn resolve(
    registry: &Registry,
    project: &Project,
    brief: &Brief<'_>,
    levels: &[String],
    usage: &Usage,
) -> Resolution {
    let mut resolution = Resolution {
        mode: Choice::new(DEFAULT_LAUNCH_MODE, Source::Default),
        model: Choice::new(DEFAULT_MODEL.to_owned(), Source::Default),
        effort: Choice::new(Some(DEFAULT_EFFORT.to_owned()), Source::Default),
        ultracode: Choice::new(false, Source::Default),
        permission_mode: permission_mode(project),
        notes: Vec::new(),
        downshift: None,
    };
    let launch = &project.launch;
    if let Some(mode) = launch.mode {
        resolution.mode = Choice::new(mode, Source::Project);
    }
    if let Some(model) = &launch.model {
        resolution.model = Choice::new(model.trim().to_owned(), Source::Project);
    }
    if let Some(effort) = &launch.effort {
        resolution.effort = Choice::new(Some(effort.trim().to_owned()), Source::Project);
    }
    let levels: Vec<String> = if levels.is_empty() {
        EFFORTS.map(str::to_owned).to_vec()
    } else {
        levels.to_vec()
    };

    let built_in = built_in();
    let (global, built): (&[Rule], bool) = if registry.rules.is_empty() {
        (&built_in, true)
    } else {
        (&registry.rules, false)
    };
    if let Some((index, rule)) = first_match(global, brief) {
        let source = if built {
            Source::BuiltIn(rule.label())
        } else {
            Source::Rule(index + 1, rule.label())
        };
        resolution.apply(&rule.set, source, &levels);
    }
    if let Some((index, rule)) = first_match(&project.rules, brief) {
        resolution.apply(
            &rule.set,
            Source::ProjectRule(index + 1, rule.label()),
            &levels,
        );
    }
    resolution.downshift(brief, usage, registry.settings.fable_downshift_percent());
    resolution
}

fn first_match<'a>(rules: &'a [Rule], brief: &Brief<'_>) -> Option<(usize, &'a Rule)> {
    rules
        .iter()
        .enumerate()
        .find(|(_, rule)| rule.when.matches(brief))
}

impl Resolution {
    fn apply(&mut self, set: &Set, source: Source, levels: &[String]) {
        if let Some(model) = &set.model {
            self.model = Choice::new(model.trim().to_owned(), source.clone());
        }
        if let Some(effort) = &set.effort {
            self.effort = Choice::new(Some(effort.trim().to_owned()), source.clone());
        }
        if let Some(by) = set.effort_step {
            match step(levels, self.effort.value.as_deref(), by) {
                Ok((level, detail)) => {
                    self.effort = Choice {
                        value: Some(level),
                        source: source.clone(),
                        detail: Some(detail),
                    };
                }
                Err(why) => self.notes.push(format!("{source}: {why}")),
            }
        }
        if let Some(ultracode) = set.ultracode {
            self.ultracode = Choice::new(ultracode, source);
        }
    }

    /// The owner's choice over the rules' (EP-19a): the model, effort and ultracode as they
    /// picked them, each with [`Source::Owner`].
    pub fn pick(&mut self, picked: &Picked) {
        self.model = Choice::new(picked.model.trim().to_owned(), Source::Owner);
        self.effort = Choice::new(
            picked
                .effort
                .as_ref()
                .map(|effort| effort.trim().to_owned()),
            Source::Owner,
        );
        self.ultracode = Choice::new(picked.ultracode, Source::Owner);
    }

    /// The last layer (D-23): an S brief whose model so far is a Fable model is proposed
    /// [`DOWNSHIFT_MODEL`] at the same effort while the Fable figure is above `mark` percent. A
    /// figure above the mark counts whatever its age, since a window's share only grows; one
    /// whose window has reset since says nothing. M and L briefs are not weighed.
    fn downshift(&mut self, brief: &Brief<'_>, usage: &Usage, mark: u8) {
        if brief.size != Some(Size::S) || !is_fable(&self.model.value) {
            return;
        }
        let judged = match &usage.fable {
            None => Downshift {
                applied: false,
                words: "no downshift: the meter has no Fable figure yet".to_owned(),
            },
            Some(fable) => {
                let age = short_age(usage.now - fable.at);
                let read = if age == "now" {
                    "read just now".to_owned()
                } else {
                    format!("read {age} ago")
                };
                let share = usage::percent(fable.limit.percent);
                if !fable.limit.before_reset(usage.now) {
                    Downshift {
                        applied: false,
                        words: format!(
                            "no downshift: Fable's last figure ({share}, {read}) is of a window \
                             that has ended"
                        ),
                    }
                } else if fable.limit.percent > f64::from(mark) {
                    Downshift {
                        applied: true,
                        words: format!("Fable at {share}, {read}, above {mark} %"),
                    }
                } else if usage.now - fable.at > usage::FRESH_FOR {
                    Downshift {
                        applied: false,
                        words: format!(
                            "Fable at {share}, {read}: under the downshift's {mark} %, though \
                             it may be higher now"
                        ),
                    }
                } else {
                    Downshift {
                        applied: false,
                        words: format!("Fable at {share}, {read}: under the downshift's {mark} %"),
                    }
                }
            }
        };
        if judged.applied {
            self.model = Choice {
                value: DOWNSHIFT_MODEL.to_owned(),
                source: Source::Downshift,
                detail: Some(judged.words.clone()),
            };
        }
        self.downshift = Some(judged);
    }
}

/// The level `by` steps from `from` along `levels`, held at the ends, and the step in words.
fn step(levels: &[String], from: Option<&str>, by: i64) -> Result<(String, String), String> {
    let from = from.ok_or("no effort to step from")?;
    let at = levels
        .iter()
        .position(|level| level == from)
        .ok_or_else(|| format!("{from} is not among the levels {}", levels.join(", ")))?;
    let last = levels.len() as i64 - 1;
    let to = (at as i64 + by).clamp(0, last);
    let steps = match by.abs() {
        1 => "one step".to_owned(),
        n => format!("{n} steps"),
    };
    let way = if by < 0 { "lower" } else { "higher" };
    let mut detail = format!("{steps} {way} than {from}");
    if to != at as i64 + by {
        detail.push_str(if by < 0 {
            ", held at the lowest"
        } else {
            ", held at the highest"
        });
    }
    Ok((levels[to as usize].clone(), detail))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::registry::State;

    fn registry(text: &str) -> Registry {
        let registry: Registry = toml::from_str(text).unwrap();
        registry.validate().unwrap();
        registry
    }

    const PROJECT: &str = "format = 1\n\n[[project]]\nname = \"alpha\"\npath = \"alpha\"\n\
                           state = \"active\"\n";

    fn brief(size: Option<Size>, phase: &'static str, title: &'static str) -> Brief<'static> {
        Brief {
            size,
            phase: Some(phase),
            title,
        }
    }

    fn levels() -> Vec<String> {
        EFFORTS.map(str::to_owned).to_vec()
    }

    fn now() -> DateTime<Utc> {
        DateTime::parse_from_rfc3339("2026-11-04T12:00:00Z")
            .unwrap()
            .with_timezone(&Utc)
    }

    fn unread() -> Usage {
        Usage::unread(now())
    }

    /// The meter read Fable at `percent`, `minutes` ago, its window resetting in three days. The
    /// endpoint calls the Fable-scoped limit inactive while its window runs (seen 2026-11-05),
    /// which counts for nothing.
    fn fable(percent: f64, minutes: i64) -> Usage {
        Usage {
            fable: Some(Fable {
                limit: Limit {
                    percent,
                    resets_at: Some(now() + chrono::TimeDelta::days(3)),
                    active: Some(false),
                    severity: None,
                },
                at: now() - chrono::TimeDelta::minutes(minutes),
            }),
            now: now(),
        }
    }

    #[test]
    fn the_defaults_and_the_built_in_table() {
        let registry = registry(PROJECT);
        let project = &registry.projects[0];
        assert_eq!(project.state, State::Active);
        let m = resolve(
            &registry,
            project,
            &brief(Some(Size::M), "P2", "Dialog"),
            &[],
            &unread(),
        );
        assert_eq!(
            (m.model.value.as_str(), m.effort.value.as_deref()),
            ("fable", Some("xhigh"))
        );
        assert_eq!(m.model.reason(), "the tool's default");
        assert_eq!(m.permission_mode.value, "acceptEdits");
        assert!(!m.ultracode.value);
        assert_eq!(m.mode.value, DEFAULT_LAUNCH_MODE);
        assert_eq!(m.mode.reason(), "the tool's default");
        assert_eq!(m.downshift, None, "an M brief is not weighed");
        // An S brief, with no global rule written: the built-in table steps one lower.
        let s = resolve(
            &registry,
            project,
            &brief(Some(Size::S), "P2", "Small"),
            &[],
            &unread(),
        );
        assert_eq!(s.effort.value.as_deref(), Some("high"));
        assert_eq!(
            s.effort.reason(),
            "the built-in rule: an S brief runs one effort step lower (D-22) \
             (one step lower than xhigh)"
        );
        assert!(s.notes.is_empty());
        // No reading yet: Fable stays, and the downshift says why it could not judge.
        assert_eq!(s.model.value, "fable");
        let judged = s.downshift.unwrap();
        assert!(!judged.applied);
        assert_eq!(
            judged.words,
            "no downshift: the meter has no Fable figure yet"
        );
    }

    #[test]
    fn the_downshift_proposes_opus_for_an_s_brief_above_the_mark() {
        let registry = registry(PROJECT);
        let project = &registry.projects[0];
        let small = brief(Some(Size::S), "P3", "Usage coupling");
        let s = resolve(&registry, project, &small, &[], &fable(64.0, 3));
        assert_eq!(s.model.value, DOWNSHIFT_MODEL);
        assert_eq!(s.model.source, Source::Downshift);
        assert_eq!(
            s.model.reason(),
            "the Fable downshift (D-23) (Fable at 64 %, read 3 min ago, above 60 %)"
        );
        // The same effort as without it: the built-in step's.
        assert_eq!(s.effort.value.as_deref(), Some("high"));
        assert!(s.downshift.as_ref().is_some_and(|d| d.applied));

        // At the mark and under it, Fable stays; an old figure under it may have grown.
        for (percent, minutes, words) in [
            (
                60.0,
                3,
                "Fable at 60 %, read 3 min ago: under the downshift's 60 %",
            ),
            (
                19.0,
                0,
                "Fable at 19 %, read just now: under the downshift's 60 %",
            ),
            (
                40.0,
                120,
                "Fable at 40 %, read 2 h ago: under the downshift's 60 %, though it may be \
                 higher now",
            ),
        ] {
            let s = resolve(&registry, project, &small, &[], &fable(percent, minutes));
            assert_eq!(s.model.value, "fable", "{percent}");
            assert_eq!(
                s.downshift,
                Some(Downshift {
                    applied: false,
                    words: words.to_owned()
                })
            );
        }
        // A figure above the mark counts whatever its age: a window's share only grows.
        let old = resolve(&registry, project, &small, &[], &fable(75.0, 600));
        assert_eq!(old.model.value, DOWNSHIFT_MODEL);
        assert!(old.model.reason().contains("read 10 h ago"));

        // A figure whose window has ended says nothing.
        let mut ended = fable(90.0, 30);
        if let Some(fable) = &mut ended.fable {
            fable.limit.resets_at = Some(now() - chrono::TimeDelta::minutes(5));
        }
        let s = resolve(&registry, project, &small, &[], &ended);
        assert_eq!(s.model.value, "fable");
        assert_eq!(
            s.downshift.unwrap().words,
            "no downshift: Fable's last figure (90 %, read 30 min ago) is of a window that has \
             ended"
        );

        // M and L keep Fable, unweighed.
        for size in [Size::M, Size::L] {
            let big = resolve(
                &registry,
                project,
                &brief(Some(size), "P3", "Engine"),
                &[],
                &fable(95.0, 1),
            );
            assert_eq!(big.model.value, "fable");
            assert_eq!(big.downshift, None);
        }
    }

    #[test]
    fn the_downshift_weighs_only_fable_and_takes_the_registrys_mark() {
        // The project's launch defaults choose Opus: nothing to downshift.
        let opus = registry(&format!(
            "{PROJECT}\n[project.launch]\nmodel = \"claude-opus-5-5\"\n"
        ));
        let s = resolve(
            &opus,
            &opus.projects[0],
            &brief(Some(Size::S), "P3", "x"),
            &[],
            &fable(90.0, 1),
        );
        assert_eq!(
            (s.model.value.as_str(), s.model.source.clone()),
            ("claude-opus-5-5", Source::Project)
        );
        assert_eq!(s.downshift, None);

        // A project rule's Fable id is downshifted too, and the registry's mark is the one used.
        let text = format!(
            "{PROJECT}\n[[project.rule]]\nmatch = {{ size = \"S\" }}\n\
             set = {{ model = \"claude-fable-5-1\", effort = \"max\" }}\n"
        )
        .replacen(
            "format = 1\n",
            "format = 1\n\n[settings]\nfable_downshift_percent = 80\n",
            1,
        );
        let marked = registry(&text);
        let small = brief(Some(Size::S), "P3", "x");
        let under = resolve(&marked, &marked.projects[0], &small, &[], &fable(75.0, 1));
        assert_eq!(under.model.value, "claude-fable-5-1");
        assert!(
            under
                .downshift
                .unwrap()
                .words
                .contains("under the downshift's 80 %")
        );
        let over = resolve(&marked, &marked.projects[0], &small, &[], &fable(81.0, 1));
        assert_eq!(over.model.value, DOWNSHIFT_MODEL);
        // The project rule's effort stays: the same effort, another model.
        assert_eq!(over.effort.value.as_deref(), Some("max"));
        assert!(is_fable("fable") && is_fable("claude-fable-5-1") && !is_fable("opus"));
    }

    #[test]
    fn layers_from_the_default_to_the_projects_rule() {
        let text = format!(
            "{PROJECT}\n[project.launch]\nmode = \"headless\"\nmodel = \"claude-opus-5-5\"\n\
             effort = \"high\"\npermission_mode = \"auto\"\n\n\
             [[project.rule]]\nmatch = {{ title = [\"Re-plan\"] }}\nset = {{ effort = \"max\" }}\n"
        );
        let text = text.replacen(
            "format = 1\n",
            "format = 1\n\n[[rule]]\nname = \"large briefs on Fable\"\n\
             match = { size = \"L\" }\nset = { model = \"fable\", ultracode = true }\n\n\
             [[rule]]\nmatch = { size = [\"S\", \"M\"], phase = \"p2\" }\n\
             set = { effort_step = 1 }\n",
            1,
        );
        let registry = registry(&text);
        let project = &registry.projects[0];

        let l = resolve(
            &registry,
            project,
            &brief(Some(Size::L), "P3", "Engine"),
            &levels(),
            &unread(),
        );
        assert_eq!(l.model.value, "fable");
        assert_eq!(l.model.reason(), "rule 1: large briefs on Fable");
        assert!(l.ultracode.value);
        // The project's own effort holds where no rule sets one.
        assert_eq!(l.effort.value.as_deref(), Some("high"));
        assert_eq!(l.effort.source, Source::Project);
        assert_eq!(l.permission_mode.value, "auto");
        // The project's launch mode holds over the tool's; no rule sets one.
        assert_eq!(
            (l.mode.value, l.mode.source.clone()),
            (LaunchMode::Headless, Source::Project)
        );

        // A written global rule replaces the built-in table, steps from the project's level.
        let s = resolve(
            &registry,
            project,
            &brief(Some(Size::S), "P2", "Small"),
            &levels(),
            &unread(),
        );
        assert_eq!(s.model.value, "claude-opus-5-5");
        assert_eq!(s.effort.value.as_deref(), Some("xhigh"));
        assert_eq!(
            s.effort.reason(),
            "rule 2: size S or M, phase p2 (one step higher than high)"
        );

        // The project's rule comes last and wins.
        let replan = resolve(
            &registry,
            project,
            &brief(Some(Size::S), "P2", "Re-plan P2 and the switch"),
            &levels(),
            &unread(),
        );
        assert_eq!(replan.effort.value.as_deref(), Some("max"));
        assert_eq!(
            replan.effort.reason(),
            "the project's rule 1: title \"Re-plan\""
        );
    }

    /// The project's permission mode (EP-19b), the one layer of it: its own, else the tool's.
    #[test]
    fn the_permission_mode_is_the_projects_else_the_tools() {
        let registry = registry(PROJECT);
        let mode = permission_mode(&registry.projects[0]);
        assert_eq!(mode.value, DEFAULT_PERMISSION_MODE);
        assert_eq!(mode.reason(), "the tool's default");
        let trusted = self::registry(&format!(
            "{PROJECT}\n[project.launch]\npermission_mode = \" auto \"\n"
        ));
        let mode = permission_mode(&trusted.projects[0]);
        assert_eq!(mode.value, "auto");
        assert_eq!(mode.reason(), "the project's launch defaults");
        // The resolution takes it as it is.
        let resolved = resolve(
            &trusted,
            &trusted.projects[0],
            &brief(Some(Size::M), "P2", "x"),
            &[],
            &unread(),
        );
        assert_eq!(resolved.permission_mode, mode);
        // Every mode the gate waits in is one the tool knows, but the CLI's `default`.
        for waiting in crate::autopilot::WAITING_MODES {
            assert!(
                waiting == "default" || PERMISSION_MODES.contains(&waiting),
                "{waiting}"
            );
        }
        assert!(PERMISSION_MODES.contains(&DEFAULT_PERMISSION_MODE));
    }

    #[test]
    fn steps_hold_at_the_ends_and_say_what_they_could_not_do() {
        let levels = levels();
        assert_eq!(
            step(&levels, Some("low"), -1),
            Ok((
                "low".to_owned(),
                "one step lower than low, held at the lowest".to_owned()
            ))
        );
        assert_eq!(
            step(&levels, Some("high"), 5),
            Ok((
                "max".to_owned(),
                "5 steps higher than high, held at the highest".to_owned()
            ))
        );
        assert!(step(&levels, None, -1).is_err());
        assert!(
            step(&levels, Some("ultra"), -1)
                .unwrap_err()
                .contains("not among")
        );

        // A model whose levels lack the effort: the step is skipped and said.
        let registry = registry(&format!(
            "{PROJECT}\n[project.launch]\neffort = \"xhigh\"\n"
        ));
        let short = ["low", "medium", "high", "max"].map(str::to_owned);
        let s = resolve(
            &registry,
            &registry.projects[0],
            &brief(Some(Size::S), "P1", "x"),
            &short,
            &unread(),
        );
        assert_eq!(s.effort.value.as_deref(), Some("xhigh"));
        assert_eq!(s.notes.len(), 1);
        assert!(s.notes[0].contains("not among"), "{:?}", s.notes);
    }

    #[test]
    fn matching_and_describing() {
        let rule = |text: &str| -> Rule { toml::from_str(text).unwrap() };
        let any = rule("set = { effort = \"low\" }");
        assert!(any.when.matches(&Brief::default()));
        assert_eq!(any.label(), "every brief");
        let titled = rule("match = { title = \"re-plan\" }\nset = { effort = \"low\" }");
        assert!(titled.when.matches(&brief(None, "P1", "Re-Plan P1: retro")));
        assert!(!titled.when.matches(&brief(None, "P1", "Replanning")));
        let sized = rule("match = { size = \"s\" }\nset = { effort = \"low\" }");
        assert!(!sized.when.matches(&brief(None, "P1", "no size")));
        assert!(sized.when.matches(&brief(Some(Size::S), "P1", "x")));
        assert_eq!(sized.when.describe(), "size s");
    }

    #[test]
    fn refuses_rules_that_cannot_be_meant() {
        for (set, message) in [
            (
                "match = { size = \"XL\" }\nset = { effort = \"low\" }",
                "S, M or L",
            ),
            ("set = {}", "sets nothing"),
            ("set = { effort = \"low\", effort_step = -1 }", "not both"),
            ("set = { effort_step = 0 }", "changes nothing"),
            ("set = { model = \" \" }", "model is empty"),
            (
                "match = { title = [\"\"] }\nset = { effort = \"low\" }",
                "empty word",
            ),
        ] {
            let rule: Rule = toml::from_str(set).unwrap();
            let error = rule.check().unwrap_err();
            assert!(error.contains(message), "{set}: {error}");
        }
        assert!(toml::from_str::<Rule>("when = { size = \"S\" }").is_err());
        assert!(toml::from_str::<Rule>("set = { effrot = \"low\" }").is_err());
    }
}
