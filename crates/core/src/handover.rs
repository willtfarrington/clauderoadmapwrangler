//! The handover (D-22, D-29; EP-37a): what the window carries across its restart into the
//! installed build. Before the window starts the installed window (`--restarted <pid>`), it writes
//! its held chains and chain plans, whether the new window opens hidden, the urgent messages it
//! has seen (EP-20c) and the project whose launch dialog to open again (EP-39) to [`FILE`] in the
//! local folder; the restarted window takes the file at its start, which reads and deletes it. A
//! file another process wrote, of another version, or that does not read is deleted too, and what
//! it held is named, so that a later start never takes up an old chain. The old window writes the
//! file and the new build reads it: a build that changes [`Held`], [`Plan`] or what a plan's rows
//! carry keeps the old form readable (a new field with `#[serde(default)]`) or raises [`VERSION`].

use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::thread;
use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::autopilot::{Held, Plan};
use crate::{files, process};

/// The handover's file in the local folder (`CRW_HOME` in a trial).
pub const FILE: &str = "handover.json";
/// The form this build writes and reads.
pub const VERSION: u32 = 1;

/// What the window carries across its restart.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Handover {
    pub version: u32,
    /// The old window's process id, which the restarted window is started with.
    pub from: u32,
    /// When it was written, in RFC 3339.
    pub written: String,
    pub held: Vec<Held>,
    pub plans: Vec<Plan>,
    /// The new window opens hidden in the tray: the old one was, and the restart was the gate's.
    #[serde(default)]
    pub hidden: bool,
    /// The urgent messages the old window had seen at its last look (EP-20c): the new window's
    /// first look toasts what arrived since, instead of taking every open one as seen. None
    /// when the old window had not looked, or wrote the form before this field.
    #[serde(default)]
    pub urgent_seen: Option<Vec<UrgentSeen>>,
    /// The project whose launch dialog the new window opens again, once its files are read
    /// (EP-39): the dialog's own Install and restart closed it for the restart. None when no
    /// dialog asked, or the form was written before this field.
    #[serde(default)]
    pub open_launch: Option<String>,
}

/// An urgent message a window has seen: its project and its id (EP-20c).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UrgentSeen {
    pub project: String,
    pub id: String,
}

impl Handover {
    /// The handover of the window whose process is `from`, written at `written`.
    pub fn new(
        from: u32,
        written: String,
        held: Vec<Held>,
        plans: Vec<Plan>,
        hidden: bool,
        urgent_seen: Option<Vec<UrgentSeen>>,
        open_launch: Option<String>,
    ) -> Self {
        Self {
            version: VERSION,
            from,
            written,
            held,
            plans,
            hidden,
            urgent_seen,
            open_launch,
        }
    }

    /// Whether anything in it needs the file: a held chain, a plan, a hidden window or a dialog
    /// to open again. A window with nothing to carry restarts without one.
    pub fn carries(&self) -> bool {
        !self.held.is_empty() || !self.plans.is_empty() || self.hidden || self.open_launch.is_some()
    }

    /// What it carries, in words: `1 held chain (harbor) and 2 chain plans (harbor, meadow)`,
    /// `the launch dialog of harbor` (EP-39).
    pub fn words(&self) -> String {
        let count = |n: usize, one: &str, many: &str, names: Vec<&str>| {
            let what = if n == 1 { one } else { many };
            format!("{n} {what} ({})", names.join(", "))
        };
        let mut parts = Vec::new();
        if !self.held.is_empty() {
            let names = self.held.iter().map(|held| held.project.as_str()).collect();
            parts.push(count(self.held.len(), "held chain", "held chains", names));
        }
        if !self.plans.is_empty() {
            let names = self
                .plans
                .iter()
                .map(|plan| plan.project.as_str())
                .collect();
            parts.push(count(self.plans.len(), "chain plan", "chain plans", names));
        }
        if let Some(project) = &self.open_launch {
            parts.push(format!("the launch dialog of {project}"));
        }
        if parts.is_empty() {
            "no chain".to_owned()
        } else {
            parts.join(" and ")
        }
    }
}

/// The handover's path in `dir`, the local folder.
pub fn path(dir: &Path) -> PathBuf {
    dir.join(FILE)
}

/// Writes `handover` into `dir` through a temporary file, so that a reader never meets half of it.
pub fn write(dir: &Path, handover: &Handover) -> Result<PathBuf, String> {
    let path = path(dir);
    let bytes = serde_json::to_vec_pretty(handover)
        .map_err(|e| format!("the handover did not serialize: {e}"))?;
    fs::create_dir_all(dir)
        .and_then(|()| files::write_atomic(&path, &bytes))
        .map_err(|e| format!("{} was not written: {e}", path.display()))?;
    Ok(path)
}

/// What a window's start found in the local folder.
#[derive(Debug, Clone, PartialEq)]
pub enum Taken {
    /// No handover.
    Nothing,
    /// The handover the old window wrote for this one.
    Ours(Handover),
    /// A file another process wrote, of another version, or that does not read, deleted: what it
    /// held, in words.
    Left(String),
}

/// Takes the handover in `dir` at a window's start: read and deleted. `from` is the old window's
/// process id when the window was started with `--restarted <pid>`; only its file is taken.
/// Another restart's file whose old window still runs is left for the window that restart
/// starts (no single instance until EP-28, so a window started by hand meanwhile leaves it).
pub fn take(dir: &Path, from: Option<u32>) -> Taken {
    take_with(dir, from, process::running)
}

/// [`take`], with `running` saying whether a writer's process still runs (a test's seam).
pub fn take_with(dir: &Path, from: Option<u32>, running: impl Fn(u32) -> bool) -> Taken {
    let path = path(dir);
    let bytes = match fs::read(&path) {
        Ok(bytes) => bytes,
        Err(e) if e.kind() == ErrorKind::NotFound => return Taken::Nothing,
        Err(e) => {
            return Taken::Left(format!(
                "{} did not read ({e}){}",
                path.display(),
                removed(&path)
            ));
        }
    };
    let parsed = serde_json::from_slice::<Handover>(&bytes);
    if let Ok(found) = &parsed
        && found.version == VERSION
        && Some(found.from) != from
        && running(found.from)
    {
        return Taken::Nothing;
    }
    // Taken or not, the file goes: an old chain never comes back. One this window takes but
    // cannot delete is carried all the same, and a later start names it.
    let deleted = removed(&path);
    match parsed {
        Err(e) => Taken::Left(format!("a handover that does not read ({e}){deleted}")),
        Ok(found) if found.version != VERSION => Taken::Left(format!(
            "a handover of version {} (this build reads {VERSION}), with {}{deleted}",
            found.version,
            found.words()
        )),
        Ok(found) if Some(found.from) != from => Taken::Left(format!(
            "a handover written by process {} at {}, with {}{deleted}",
            found.from,
            found.written,
            found.words()
        )),
        Ok(found) => Taken::Ours(found),
    }
}

/// Deletes `path`, trying again a moment later while security software holds a fresh file, and
/// says how it went: nothing when deleted, else why not.
fn removed(path: &Path) -> String {
    let mut why = String::new();
    for attempt in 0..5 {
        match fs::remove_file(path) {
            Ok(()) => return String::new(),
            Err(e) if e.kind() == ErrorKind::NotFound => return String::new(),
            Err(e) => why = e.to_string(),
        }
        if attempt < 4 {
            thread::sleep(Duration::from_millis(50));
        }
    }
    format!(" (not deleted: {why})")
}

/// Removes the handover in `dir`: the restart did not go, and the window keeps what it holds.
pub fn discard(dir: &Path) {
    let _ = fs::remove_file(path(dir));
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::autopilot::{Planned, Stop};
    use crate::files::tests::Scratch;
    use crate::registry::LaunchMode;
    use crate::roadmap::{EpId, Size};
    use crate::rules::{Choice, Downshift, Picked, Resolution, Source};

    fn ep(text: &str) -> EpId {
        text.parse().unwrap()
    }

    fn choice<T>(value: T, source: Source, detail: Option<&str>) -> Choice<T> {
        Choice {
            value,
            source,
            detail: detail.map(str::to_owned),
        }
    }

    /// A plan with a row left alone, a row the owner chose and one left out, its resolution's
    /// values from every kind of source, and where the list stopped.
    fn plan() -> Plan {
        let resolution = Resolution {
            mode: choice(LaunchMode::Headless, Source::Owner, None),
            model: choice("fable".to_owned(), Source::Default, None),
            effort: choice(
                Some("high".to_owned()),
                Source::BuiltIn("S briefs one step lower".to_owned()),
                Some("one step lower than xhigh"),
            ),
            ultracode: choice(false, Source::Rule(2, "large briefs".to_owned()), None),
            permission_mode: choice("auto".to_owned(), Source::Project, None),
            notes: vec!["ultracode is not offered by this CLI".to_owned()],
            downshift: Some(Downshift {
                applied: false,
                words: "Fable at 12 %, read 3 min ago, under 60 %".to_owned(),
            }),
        };
        let mut chosen = resolution.clone();
        chosen.model = choice(
            "claude-opus-5-5".to_owned(),
            Source::ProjectRule(1, "kept for this project".to_owned()),
            None,
        );
        let planned =
            |id: &str, chain: bool, picked: Option<Picked>, resolution: &Resolution| Planned {
                ep: ep(id),
                title: format!("Brief {id}"),
                size: Some(Size::M),
                resolution: resolution.clone(),
                due: Vec::new(),
                chain,
                picked,
            };
        Plan {
            project: "harbor".to_owned(),
            after: ep("EP-4"),
            briefs: vec![
                planned("EP-4a", true, None, &resolution),
                planned(
                    "EP-5",
                    true,
                    Some(Picked {
                        model: "claude-opus-5-5".to_owned(),
                        effort: Some("max".to_owned()),
                        ultracode: false,
                    }),
                    &chosen,
                ),
                planned("EP-6", false, None, &resolution),
            ],
            stop: Some(Stop {
                ep: ep("EP-7"),
                why: "EP-7 is a re-plan, the owner's to run".to_owned(),
            }),
            daily: 3,
        }
    }

    fn handover(from: u32) -> Handover {
        Handover::new(
            from,
            "2026-11-10T16:40:00+00:00".to_owned(),
            vec![Held {
                project: "harbor".to_owned(),
                after: Some(ep("EP-4")),
                words: "the chain is held after EP-4 for the restart".to_owned(),
            }],
            vec![plan()],
            true,
            Some(vec![UrgentSeen {
                project: "harbor".to_owned(),
                id: "IN-4".to_owned(),
            }]),
            None,
        )
    }

    #[test]
    fn the_old_windows_handover_is_taken_once_as_written() {
        let scratch = Scratch::new("handover-take");
        assert_eq!(take(&scratch.0, Some(4242)), Taken::Nothing);
        let written = handover(4242);
        let path = write(&scratch.0, &written).unwrap();
        assert_eq!(path, scratch.0.join(FILE));
        // The JSON keeps ids by their text.
        let text = fs::read_to_string(&path).unwrap();
        assert!(text.contains("\"EP-4a\""), "{text}");
        assert!(text.contains("\"urgent_seen\""), "{text}");
        assert_eq!(take(&scratch.0, Some(4242)), Taken::Ours(written.clone()));
        assert!(!path.exists(), "taken means deleted");
        assert_eq!(take(&scratch.0, Some(4242)), Taken::Nothing);
        assert_eq!(
            written.words(),
            "1 held chain (harbor) and 1 chain plan (harbor)"
        );
        assert!(written.carries());
        // A dialog to open again is carried on its own (EP-39), and named.
        let dialog = Handover::new(1, String::new(), Vec::new(), Vec::new(), false, None, None);
        assert!(!dialog.carries());
        let dialog = Handover {
            open_launch: Some("meadow".to_owned()),
            ..dialog
        };
        assert!(dialog.carries());
        assert_eq!(dialog.words(), "the launch dialog of meadow");
        let mut both = written.clone();
        both.open_launch = Some("meadow".to_owned());
        assert_eq!(
            both.words(),
            "1 held chain (harbor) and 1 chain plan (harbor) and the launch dialog of meadow"
        );
    }

    #[test]
    fn a_file_left_by_another_window_or_unreadable_is_deleted_and_named() {
        let scratch = Scratch::new("handover-left");
        // Another restart's file whose old window still runs is left for its own new window.
        write(&scratch.0, &handover(4242)).unwrap();
        assert_eq!(
            take_with(&scratch.0, Some(7), |pid| pid == 4242),
            Taken::Nothing
        );
        assert!(path(&scratch.0).exists());
        // Its writer gone: another process's file, and one met by a window started by hand.
        for from in [Some(7), None] {
            write(&scratch.0, &handover(4242)).unwrap();
            let Taken::Left(words) = take_with(&scratch.0, from, |_| false) else {
                panic!("another window's handover is not taken");
            };
            assert!(
                words.starts_with("a handover written by process 4242 at 2026-11-10")
                    && words.ends_with("with 1 held chain (harbor) and 1 chain plan (harbor)"),
                "{words}"
            );
            assert!(!path(&scratch.0).exists());
        }
        // Another version.
        let mut other = handover(4242);
        other.version = VERSION + 1;
        write(&scratch.0, &other).unwrap();
        let Taken::Left(words) = take(&scratch.0, Some(4242)) else {
            panic!("another version is not taken");
        };
        assert!(words.starts_with("a handover of version 2"), "{words}");
        assert!(!path(&scratch.0).exists());
        // A file that does not read.
        fs::write(path(&scratch.0), "{ half").unwrap();
        let Taken::Left(words) = take(&scratch.0, Some(4242)) else {
            panic!("a broken file is not taken");
        };
        assert!(
            words.starts_with("a handover that does not read"),
            "{words}"
        );
        assert!(!path(&scratch.0).exists());
        // An empty handover in words, and a discarded one.
        let empty = Handover::new(1, String::new(), Vec::new(), Vec::new(), false, None, None);
        assert_eq!(empty.words(), "no chain");
        write(&scratch.0, &empty).unwrap();
        discard(&scratch.0);
        assert_eq!(take(&scratch.0, Some(1)), Taken::Nothing);
    }

    /// A version 1 file as this build writes it, frozen: a later build reads it, or raises the
    /// version (the module's rule). It names every kind of source a resolution's value has.
    const FROZEN: &str = r#"{
      "version": 1, "from": 9, "written": "2026-11-10T16:40:00+00:00",
      "held": [{"project": "harbor", "after": "EP-4", "words": "held after EP-4 until the pause lifts"}],
      "plans": [{
        "project": "harbor", "after": "EP-4",
        "briefs": [{
          "ep": "EP-4a", "title": "Brief EP-4a", "size": "M",
          "resolution": {
            "mode": {"value": "headless", "source": "Owner", "detail": null},
            "model": {"value": "fable", "source": "Default", "detail": null},
            "effort": {"value": "high", "source": {"BuiltIn": "S briefs one step lower"},
                       "detail": "one step lower than xhigh"},
            "ultracode": {"value": false, "source": {"Rule": [2, "large briefs"]}, "detail": null},
            "permission_mode": {"value": "auto", "source": "Project", "detail": null},
            "notes": [],
            "downshift": {"applied": true, "words": "Fable at 64 %, above 60 %"}
          },
          "due": ["OT-3"], "chain": true,
          "picked": {"model": "claude-opus-5-5", "effort": "max", "ultracode": false}
        }, {
          "ep": "EP-5", "title": "Brief EP-5", "size": null,
          "resolution": {
            "mode": {"value": "visual", "source": "Project", "detail": null},
            "model": {"value": "claude-opus-5-5",
                      "source": {"ProjectRule": [1, "kept for this project"]}, "detail": null},
            "effort": {"value": null, "source": "Downshift", "detail": null},
            "ultracode": {"value": false, "source": "Default", "detail": null},
            "permission_mode": {"value": "acceptEdits", "source": "Default", "detail": null},
            "notes": ["ultracode is not offered by this CLI"],
            "downshift": null
          },
          "due": [], "chain": false, "picked": null
        }],
        "stop": {"ep": "EP-7", "why": "EP-7 is a re-plan, the owner's to run"},
        "daily": 3
      }],
      "hidden": true
    }"#;

    #[test]
    fn a_frozen_version_one_file_still_reads() {
        let scratch = Scratch::new("handover-frozen");
        fs::write(path(&scratch.0), FROZEN).unwrap();
        let Taken::Ours(found) = take(&scratch.0, Some(9)) else {
            panic!("a version 1 file reads");
        };
        assert!(found.hidden);
        assert_eq!(found.urgent_seen, None, "the form before EP-20c's field");
        assert_eq!(found.open_launch, None, "the form before EP-39's field");
        assert_eq!(found.held[0].after, Some(ep("EP-4")));
        let plan = &found.plans[0];
        assert_eq!(plan.briefs[0].ep, ep("EP-4a"));
        assert_eq!(
            plan.briefs[0].resolution.ultracode.source,
            Source::Rule(2, "large briefs".to_owned())
        );
        assert_eq!(
            plan.briefs[1].resolution.model.source,
            Source::ProjectRule(1, "kept for this project".to_owned())
        );
        assert_eq!(plan.briefs[1].size, None);
        assert!(!plan.briefs[1].chain);
        assert_eq!(plan.stop.as_ref().map(|stop| stop.ep), Some(ep("EP-7")));
        assert_eq!(
            plan.answer(ep("EP-4a")),
            crate::autopilot::Answer::Picked(Picked {
                model: "claude-opus-5-5".to_owned(),
                effort: Some("max".to_owned()),
                ultracode: false,
            })
        );
    }

    #[test]
    fn a_file_without_the_hidden_flag_reads_as_shown() {
        let scratch = Scratch::new("handover-default");
        fs::write(
            path(&scratch.0),
            r#"{"version": 1, "from": 9, "written": "now", "held": [], "plans": []}"#,
        )
        .unwrap();
        let Taken::Ours(found) = take(&scratch.0, Some(9)) else {
            panic!("a file without the flag reads");
        };
        assert!(!found.hidden);
        assert_eq!(found.urgent_seen, None);
        // An empty seen set is one the old window had: it reads as such, not as none (EP-20c).
        fs::write(
            path(&scratch.0),
            r#"{"version": 1, "from": 9, "written": "now", "held": [], "plans": [],
               "urgent_seen": []}"#,
        )
        .unwrap();
        let Taken::Ours(found) = take(&scratch.0, Some(9)) else {
            panic!("a file with an empty seen set reads");
        };
        assert_eq!(found.urgent_seen, Some(Vec::new()));
    }

    /// The frozen version 1 file with EP-39's field reads with it, and this build writes the
    /// field in the version 1 form, so that the build before it reads the file too (serde leaves
    /// a field it does not know alone).
    #[test]
    fn the_frozen_file_reads_with_and_without_the_dialog_to_open_again() {
        let scratch = Scratch::new("handover-frozen-launch");
        let with_field = FROZEN.replacen(
            "\"hidden\": true",
            "\"hidden\": true, \"open_launch\": \"harbor\"",
            1,
        );
        assert_ne!(with_field, FROZEN);
        fs::write(path(&scratch.0), with_field).unwrap();
        let Taken::Ours(found) = take(&scratch.0, Some(9)) else {
            panic!("a version 1 file with the field reads");
        };
        assert_eq!(found.open_launch.as_deref(), Some("harbor"));
        assert_eq!(found.version, VERSION);
        assert!(found.hidden && found.urgent_seen.is_none());
        // Written by this build: version 1 still, the field by its name.
        let mut written = handover(9);
        written.open_launch = Some("meadow".to_owned());
        let path = write(&scratch.0, &written).unwrap();
        let text = fs::read_to_string(&path).unwrap();
        assert!(text.contains("\"version\": 1"), "{text}");
        assert!(text.contains("\"open_launch\": \"meadow\""), "{text}");
        assert_eq!(take(&scratch.0, Some(9)), Taken::Ours(written));
    }
}
