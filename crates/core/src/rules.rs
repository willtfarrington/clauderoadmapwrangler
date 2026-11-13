//! The rule table (D-22): the model, effort and ultracode a brief launches with, resolved in layers,
//! each value with the reason it was chosen, which the launch dialog shows beside it: the tool's
//! default (`fable` at `xhigh`), the project's launch defaults, the global rules that match the
//! brief, then the project's own rules that match it. A rule matches by size, by phase and by
//! keywords of the title (every condition it gives must hold), and sets a model, an effort or a
//! step along the effort levels, and ultracode. Rules combine key by key (D-22's EP-19c addendum):
//! within each table, every rule that matches the brief applies, from the last to the first, so
//! the earliest that sets a key wins and a step moves from what the later rules and the layers
//! below gave. While the registry lists no global rule, the built-in table stands in for it: an S
//! brief one effort step lower. The permission mode is the project's, else `acceptEdits` (D-18),
//! and so is the launch mode, else the tool's (EP-11). Last comes the downshift (D-23; EP-18): a
//! brief of any size (S alone until EP-40) that would run on Fable is proposed [`DOWNSHIFT_MODEL`]
//! at the same effort while the meter's Fable figure is above the registry's mark. The window's
//! Keep menus and `crw set` write
//! the project's every-brief and phase rules through `registry::Changes::keep` (EP-19c), which
//! finds them by their match ([`Scope`]); the owner writes the rest by hand (rule 13).

use std::fmt;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

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

/// The model the downshift proposes for a brief on Fable while Fable's weekly share is high
/// (D-23).
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
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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

/// How the downshift judged a brief it weighs: one whose model so far is a Fable model (an S
/// brief alone until EP-40, every size since).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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

/// A brief's launch values, each with its reason; a chain plan's rows carry one across the
/// window's restart in the handover (EP-37a).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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
/// weighs, last of the layers. Within the global table (or the built-in one standing in for it),
/// then within the project's, every rule that matches the brief applies, from the last to the
/// first (D-22's EP-19c addendum): the earliest matching rule that sets a key wins, an
/// `effort_step` steps from what the later rules and the layers below gave, and each value keeps
/// the source of the rule that set it.
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
    // The steps that could not be taken, until a rule applied after them sets the effort and
    // makes them moot (EP-19c's review).
    let mut steps = Vec::new();
    for (index, rule) in matching(global, brief) {
        let source = if built {
            Source::BuiltIn(rule.label())
        } else {
            Source::Rule(index + 1, rule.label())
        };
        resolution.apply(&rule.set, source, &levels, &mut steps);
    }
    for (index, rule) in matching(&project.rules, brief) {
        resolution.apply(
            &rule.set,
            Source::ProjectRule(index + 1, rule.label()),
            &levels,
            &mut steps,
        );
    }
    resolution.notes.extend(steps);
    resolution.downshift(brief, usage, registry.settings.fable_downshift_percent());
    resolution
}

/// The rules of one table that match `brief`, each with its index from 0, from the last to the
/// first: the order they apply in, so that the earliest that sets a key wins (D-22's EP-19c
/// addendum).
fn matching<'a>(
    rules: &'a [Rule],
    brief: &'a Brief<'_>,
) -> impl Iterator<Item = (usize, &'a Rule)> {
    rules
        .iter()
        .enumerate()
        .rev()
        .filter(move |(_, rule)| rule.when.matches(brief))
}

impl Resolution {
    /// Applies one rule's `set` over the values so far. A step that cannot be taken is noted in
    /// `steps`, which an effort set by a rule applied later clears, since the step then changes
    /// nothing that runs.
    fn apply(&mut self, set: &Set, source: Source, levels: &[String], steps: &mut Vec<String>) {
        if let Some(model) = &set.model {
            self.model = Choice::new(model.trim().to_owned(), source.clone());
        }
        if let Some(effort) = &set.effort {
            self.effort = Choice::new(Some(effort.trim().to_owned()), source.clone());
            steps.clear();
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
                Err(why) => steps.push(format!("{source}: {why}")),
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

    /// The last layer (D-23): a brief whose model so far is a Fable model is proposed
    /// [`DOWNSHIFT_MODEL`] at the same effort while the Fable figure is above `mark` percent. A
    /// figure above the mark counts whatever its age, since a window's share only grows; one
    /// whose window has reset since says nothing. Every size is weighed (S alone until EP-40,
    /// the owner's choice: with Fable kept for L briefs, those are the ones to protect).
    fn downshift(&mut self, _brief: &Brief<'_>, usage: &Usage, mark: u8) {
        if !is_fable(&self.model.value) {
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

/// Where the window keeps a model or an effort (D-22; EP-19c): in the project's rule for every
/// brief, or in its rule for one phase alone. The tool finds a scope's rule by its match
/// ([`Scope::owns`]), not by its name, so a rule the owner wrote in that form counts as the
/// scope's.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Scope {
    /// Every brief of the project: the rule with no condition, which the tool writes after the
    /// project's other rules.
    Project,
    /// One phase alone, by its id as the phase headings write it: `P3`. The tool writes its rule
    /// before the every-brief rule, so that it wins for that phase's briefs.
    Phase(String),
}

impl Scope {
    /// The name the tool writes into a rule it creates, which the dialog then shows as the
    /// reason: `kept for this project`, `kept for phase P3`.
    pub fn name(&self) -> String {
        format!("kept for {}", self.words())
    }

    /// The scope in words, for the notices: `this project`, `phase P3`.
    pub fn words(&self) -> String {
        match self {
            Scope::Project => "this project".to_owned(),
            Scope::Phase(id) => format!("phase {}", id.trim()),
        }
    }

    /// Whether `rule` has this scope's form: for every brief, no condition at all; for a phase,
    /// that phase as its one condition, written as one word (trimmed, ASCII case aside), with no
    /// size and no title.
    pub fn owns(&self, rule: &Rule) -> bool {
        let when = &rule.when;
        if !when.size.is_empty() || !when.title.is_empty() {
            return false;
        }
        match self {
            Scope::Project => when.phase.is_empty(),
            Scope::Phase(id) => matches!(
                when.phase.0.as_slice(),
                [phase] if phase.trim().eq_ignore_ascii_case(id.trim())
            ),
        }
    }
}

/// What the window keeps in a rule (EP-19c): a model or an effort. Ultracode stays the rules' and
/// the owner's to write by hand.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Key {
    Model,
    Effort,
}

impl Key {
    /// The key in a rule's `set`: `model`, `effort`.
    pub fn as_str(self) -> &'static str {
        match self {
            Key::Model => "model",
            Key::Effort => "effort",
        }
    }

    /// What `set` gives for this key, trimmed; an `effort_step` is not a kept effort.
    pub fn of(self, set: &Set) -> Option<String> {
        let value = match self {
            Key::Model => &set.model,
            Key::Effort => &set.effort,
        };
        value
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_owned)
    }
}

/// The project's rule for `scope` (EP-19c): the first of its rules that `scope` owns, with its
/// index from 0 in `project.rules`, which is its place among the registry's `[[project.rule]]`
/// tables and one less than its number in a reason.
pub fn kept_rule<'a>(project: &'a Project, scope: &Scope) -> Option<(usize, &'a Rule)> {
    project
        .rules
        .iter()
        .enumerate()
        .find(|(_, rule)| scope.owns(rule))
}

/// What a project keeps for one key (EP-19c): in its rule for every brief and in its rule for the
/// brief's phase, each `None` where the rule is missing or does not set the key.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Kept {
    pub project: Option<String>,
    pub phase: Option<String>,
}

impl Kept {
    /// What `project` keeps for `key`; `phase` is the brief's phase id, when its heading has one.
    pub fn of(project: &Project, phase: Option<&str>, key: Key) -> Self {
        let kept =
            |scope: &Scope| kept_rule(project, scope).and_then(|(_, rule)| key.of(&rule.set));
        Self {
            project: kept(&Scope::Project),
            phase: phase.and_then(|id| kept(&Scope::Phase(id.to_owned()))),
        }
    }

    /// Whether the project keeps the key for every brief or for the brief's phase.
    pub fn any(&self) -> bool {
        self.project.is_some() || self.phase.is_some()
    }
}

/// What stands over a value kept in `scope`'s rule, for the brief `resolution` was resolved for
/// after the keep (EP-19c): `None` when the resolution takes its `key` from that rule, or when the
/// project has no rule for the scope; otherwise the reason of what set the key, such as `the
/// project's rule 1: kept for phase P3` or the Fable downshift's (D-23), which the dialog's notice
/// names.
pub fn stands_over(
    project: &Project,
    scope: &Scope,
    key: Key,
    resolution: &Resolution,
) -> Option<String> {
    let (index, _) = kept_rule(project, scope)?;
    let source = match key {
        Key::Model => &resolution.model.source,
        Key::Effort => &resolution.effort.source,
    };
    if matches!(source, Source::ProjectRule(n, _) if *n == index + 1) {
        return None;
    }
    Some(match key {
        Key::Model => resolution.model.reason(),
        Key::Effort => resolution.effort.reason(),
    })
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
        // Weighed since EP-40, like every brief on Fable; nothing read yet.
        assert_eq!(
            m.downshift.map(|judged| judged.applied),
            Some(false),
            "an M brief is weighed too"
        );
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

        // M and L briefs are weighed too since EP-40 (S alone before), at their own effort.
        for size in [Size::M, Size::L] {
            let big = resolve(
                &registry,
                project,
                &brief(Some(size), "P3", "Engine"),
                &[],
                &fable(95.0, 1),
            );
            assert_eq!(big.model.value, DOWNSHIFT_MODEL, "{size}");
            assert_eq!(big.effort.value.as_deref(), Some("xhigh"));
            assert!(big.downshift.is_some_and(|judged| judged.applied));
        }
        // A brief whose model is not Fable is never weighed: the next test's launch defaults.
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

    /// The project's rules in the order the window writes them (EP-19c): phase P3's effort, then
    /// the rule for every brief.
    const KEPT: &str = "\n[[project.rule]]\nname = \"kept for phase P3\"\n\
                        match = { phase = \"P3\" }\nset = { effort = \"max\" }\n\n\
                        [[project.rule]]\nname = \"kept for this project\"\n\
                        set = { model = \"claude-opus-5-5\", effort = \"high\" }\n";

    /// Rules combine key by key (D-22's EP-19c addendum): every rule that matches applies, from
    /// the last to the first, so a phase rule's effort and an every-brief rule's model both hold
    /// for a brief in that phase, and the earliest rule that sets a key wins.
    #[test]
    fn rules_combine_key_by_key_in_the_projects_table() {
        let registry = registry(&format!("{PROJECT}{KEPT}"));
        let project = &registry.projects[0];
        let resolved = |size, phase| {
            resolve(
                &registry,
                project,
                &brief(Some(size), phase, "Engine"),
                &levels(),
                &unread(),
            )
        };
        let p3 = resolved(Size::M, "P3");
        assert_eq!(p3.model.value, "claude-opus-5-5");
        assert_eq!(
            p3.model.reason(),
            "the project's rule 2: kept for this project"
        );
        // Both rules set an effort: the earlier one, the phase's, wins.
        assert_eq!(p3.effort.value.as_deref(), Some("max"));
        assert_eq!(
            p3.effort.reason(),
            "the project's rule 1: kept for phase P3"
        );
        assert!(!p3.ultracode.value);
        assert_eq!(p3.ultracode.source, Source::Default);
        // Another phase: the rule for every brief alone.
        let p2 = resolved(Size::M, "P2");
        assert_eq!(
            (p2.model.value.as_str(), p2.effort.value.as_deref()),
            ("claude-opus-5-5", Some("high"))
        );
        assert_eq!(
            p2.effort.reason(),
            "the project's rule 2: kept for this project"
        );
        // An S brief: the project's rules apply over the built-in step, the global layer.
        let s2 = resolved(Size::S, "P2");
        assert_eq!(s2.effort.value.as_deref(), Some("high"));
        assert_eq!(
            s2.effort.reason(),
            "the project's rule 2: kept for this project"
        );
        assert!(s2.notes.is_empty());
        let s3 = resolved(Size::S, "P3");
        assert_eq!(s3.effort.value.as_deref(), Some("max"));
        assert_eq!(s3.model.value, "claude-opus-5-5");
    }

    /// An earlier rule's step moves from what the later rules gave, and global rules combine as
    /// the project's do: model, effort and ultracode each from the earliest rule that sets it.
    #[test]
    fn an_earlier_step_moves_from_a_later_effort_and_global_rules_combine() {
        let text = PROJECT.replacen(
            "format = 1\n",
            "format = 1\n\n[[rule]]\nname = \"P2 lower\"\nmatch = { phase = \"P2\" }\n\
             set = { effort_step = -1 }\n\n[[rule]]\nmatch = { size = \"M\" }\n\
             set = { effort = \"high\", ultracode = true }\n\n[[rule]]\n\
             name = \"Opus throughout\"\nset = { model = \"claude-opus-5-5\", effort = \"max\" }\n",
            1,
        );
        let registry = registry(&text);
        let resolved = |registry: &Registry, size, phase| {
            resolve(
                registry,
                &registry.projects[0],
                &brief(Some(size), phase, "x"),
                &levels(),
                &unread(),
            )
        };
        // From the last rule to the first: Opus at max, then high, then one step lower.
        let m2 = resolved(&registry, Size::M, "P2");
        assert_eq!(m2.effort.value.as_deref(), Some("medium"));
        assert_eq!(
            m2.effort.reason(),
            "rule 1: P2 lower (one step lower than high)"
        );
        assert_eq!(m2.model.reason(), "rule 3: Opus throughout");
        assert!(m2.ultracode.value);
        assert_eq!(m2.ultracode.reason(), "rule 2: size M");
        // The step moves from the last rule's effort where the middle one does not match.
        let l2 = resolved(&registry, Size::L, "P2");
        assert_eq!(l2.effort.value.as_deref(), Some("xhigh"));
        assert_eq!(
            l2.effort.reason(),
            "rule 1: P2 lower (one step lower than max)"
        );
        assert!(!l2.ultracode.value);
        // And the last rule alone where the others do not.
        let l3 = resolved(&registry, Size::L, "P3");
        assert_eq!(
            (l3.model.value.as_str(), l3.effort.value.as_deref()),
            ("claude-opus-5-5", Some("max"))
        );
        assert_eq!(l3.effort.reason(), "rule 3: Opus throughout");
        // The project's table comes after the global one: its rule for every brief wins.
        let mut kept = registry.clone();
        kept.projects[0].rules = self::registry(&format!("{PROJECT}{KEPT}")).projects[0]
            .rules
            .clone();
        let over = resolved(&kept, Size::M, "P2");
        assert_eq!(over.effort.value.as_deref(), Some("high"));
        assert_eq!(
            over.effort.reason(),
            "the project's rule 2: kept for this project"
        );
        assert!(over.ultracode.value, "rule 2's ultracode stands");

        // A step that cannot be taken says so, unless a rule applied after it sets the effort
        // (EP-19c's review): here the levels lack the default's xhigh.
        let few: Vec<String> = ["low", "high"].map(str::to_owned).to_vec();
        let mut moot = self::registry(PROJECT);
        moot.projects[0].rules = vec![
            Rule {
                when: Match {
                    phase: Words(vec!["P3".to_owned()]),
                    ..Match::default()
                },
                set: Set {
                    effort: Some("max".to_owned()),
                    ..Set::default()
                },
                ..Rule::default()
            },
            Rule {
                set: Set {
                    effort_step: Some(-1),
                    ..Set::default()
                },
                ..Rule::default()
            },
        ];
        let at = |phase| {
            let brief = brief(Some(Size::M), phase, "x");
            resolve(&moot, &moot.projects[0], &brief, &few, &unread())
        };
        let p3 = at("P3");
        assert_eq!(p3.effort.value.as_deref(), Some("max"));
        assert!(p3.notes.is_empty(), "{:?}", p3.notes);
        let p2 = at("P2");
        assert_eq!(p2.notes.len(), 1);
        assert!(
            p2.notes[0].starts_with("the project's rule 2:"),
            "{:?}",
            p2.notes
        );
    }

    /// The rules the window keeps (EP-19c), found by their match: the tool's names and the
    /// owner's own rules in the same form count alike; a rule with another condition does not.
    #[test]
    fn the_kept_rules_are_found_by_their_match() {
        let text = format!(
            "{PROJECT}\n[[project.rule]]\nname = \"small in P3\"\n\
             match = {{ phase = \"P3\", size = \"S\" }}\nset = {{ model = \"fable\" }}\n\n\
             [[project.rule]]\nname = \"two phases\"\nmatch = {{ phase = [\"P3\", \"P4\"] }}\n\
             set = {{ effort = \"low\" }}\n\n[[project.rule]]\nname = \"re-plans\"\n\
             match = {{ title = \"re-plan\" }}\nset = {{ effort = \"max\" }}\n\n\
             [[project.rule]]\nname = \"mine for p3\"\nmatch = {{ phase = [\" p3 \"] }}\n\
             set = {{ effort = \"high\", ultracode = true }}\n\n[[project.rule]]\n\
             name = \"lower everywhere\"\nset = {{ effort_step = -1 }}\n\n[[project.rule]]\n\
             name = \"kept for this project\"\nset = {{ model = \"claude-opus-5-5\" }}\n"
        );
        let registry = registry(&text);
        let project = &registry.projects[0];
        let p3 = Scope::Phase("P3".to_owned());
        assert_eq!(
            kept_rule(project, &p3).map(|(index, rule)| (index, rule.label())),
            Some((3, "mine for p3".to_owned()))
        );
        // The first rule with no condition, whatever its name.
        assert_eq!(
            kept_rule(project, &Scope::Project).map(|(index, rule)| (index, rule.label())),
            Some((4, "lower everywhere".to_owned()))
        );
        assert_eq!(kept_rule(project, &Scope::Phase("P4".to_owned())), None);
        // Its step is not a kept effort, and it keeps no model: the later rule is not its.
        assert_eq!(
            Kept::of(project, Some("P3"), Key::Effort),
            Kept {
                project: None,
                phase: Some("high".to_owned())
            }
        );
        assert_eq!(Kept::of(project, Some("P3"), Key::Model), Kept::default());
        assert!(!Kept::of(project, Some("P3"), Key::Model).any());
        // The phase id in any case; no phase, or one without a rule, keeps nothing for it.
        assert!(Kept::of(project, Some("p3"), Key::Effort).any());
        assert_eq!(Kept::of(project, None, Key::Effort), Kept::default());
        assert_eq!(Kept::of(project, Some("P2"), Key::Effort), Kept::default());

        // The tool's own rules, as it writes them.
        let kept = self::registry(&format!("{PROJECT}{KEPT}"));
        let project = &kept.projects[0];
        assert_eq!(kept_rule(project, &p3).map(|(index, _)| index), Some(0));
        assert_eq!(
            kept_rule(project, &Scope::Project).map(|(index, _)| index),
            Some(1)
        );
        assert_eq!(
            Kept::of(project, Some("P3"), Key::Effort),
            Kept {
                project: Some("high".to_owned()),
                phase: Some("max".to_owned())
            }
        );
        assert_eq!(
            Kept::of(project, Some("P3"), Key::Model).project.as_deref(),
            Some("claude-opus-5-5")
        );
        // No rules at all.
        let bare = self::registry(PROJECT);
        assert_eq!(kept_rule(&bare.projects[0], &Scope::Project), None);
        assert!(!Kept::of(&bare.projects[0], Some("P3"), Key::Model).any());

        // The names the tool writes, and the words of the notices.
        assert_eq!(Scope::Project.name(), "kept for this project");
        assert_eq!(p3.name(), "kept for phase P3");
        assert_eq!(
            (Scope::Project.words(), p3.words()),
            ("this project".to_owned(), "phase P3".to_owned())
        );
        assert_eq!(
            (Key::Model.as_str(), Key::Effort.as_str()),
            ("model", "effort")
        );
        let set = Set {
            model: Some(" opus ".to_owned()),
            effort_step: Some(-1),
            ..Set::default()
        };
        assert_eq!(
            (Key::Model.of(&set), Key::Effort.of(&set)),
            (Some("opus".to_owned()), None)
        );
    }

    /// What stands over a kept value (EP-19c): nothing when the resolution takes it from the
    /// scope's rule, else the reason of what does.
    #[test]
    fn what_stands_over_a_kept_value_is_named() {
        let registry = registry(
            &format!("{PROJECT}{KEPT}").replace("model = \"claude-opus-5-5\"", "model = \"fable\""),
        );
        let project = &registry.projects[0];
        let p3 = Scope::Phase("P3".to_owned());
        let in_p3 = resolve(
            &registry,
            project,
            &brief(Some(Size::M), "P3", "x"),
            &[],
            &unread(),
        );
        // In P3, the phase's effort stands over the project's; the model is the project's.
        assert_eq!(
            stands_over(project, &Scope::Project, Key::Effort, &in_p3).as_deref(),
            Some("the project's rule 1: kept for phase P3")
        );
        assert_eq!(stands_over(project, &p3, Key::Effort, &in_p3), None);
        assert_eq!(
            stands_over(project, &Scope::Project, Key::Model, &in_p3),
            None
        );
        // Elsewhere, the project's effort holds; a phase with no rule has nothing stood over.
        let in_p2 = resolve(
            &registry,
            project,
            &brief(Some(Size::M), "P2", "x"),
            &[],
            &unread(),
        );
        assert_eq!(
            stands_over(project, &Scope::Project, Key::Effort, &in_p2),
            None
        );
        assert_eq!(
            stands_over(project, &Scope::Phase("P2".to_owned()), Key::Effort, &in_p2),
            None
        );
        // An S brief on a kept Fable while its share is high: the downshift stands over it.
        let small = resolve(
            &registry,
            project,
            &brief(Some(Size::S), "P2", "x"),
            &[],
            &fable(64.0, 3),
        );
        assert_eq!(small.model.value, DOWNSHIFT_MODEL);
        assert_eq!(
            stands_over(project, &Scope::Project, Key::Model, &small).as_deref(),
            Some("the Fable downshift (D-23) (Fable at 64 %, read 3 min ago, above 60 %)")
        );
        // The rule for every brief keeps its effort over the built-in step.
        assert_eq!(small.effort.value.as_deref(), Some("high"));
        assert_eq!(
            stands_over(project, &Scope::Project, Key::Effort, &small),
            None
        );
    }
}
