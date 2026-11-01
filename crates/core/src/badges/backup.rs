//! Backup age (D-24): the owner's Documents backup job, read and never run or changed. Its
//! settings file (a PowerShell data file: `Target`, `Sets`, `ExcludePaths`) says where the
//! archives go and which set makes one unit per folder of the data root; its run log (one JSON
//! line per event) says when each unit was last written or found unchanged, and how the last run
//! ended; the target holds `<Target>\<set>\<folder>\<yyyy-MM-dd HHmmss> <folder>.zip` with the
//! newest archive always kept. The two files are registry settings (`backup_settings`,
//! `backup_log`); without the first the badge stays empty. No child runs.

use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{Read, Seek, SeekFrom};
use std::path::{Component, Path, PathBuf};

use chrono::{DateTime, Local, NaiveDateTime, TimeDelta, TimeZone, Utc};

use super::psdata::{self, Value};
use super::rfc3339;

/// A unit the job has not written or found unchanged for this long is stale: the job runs daily,
/// so two missed days.
pub const STALE_AFTER: TimeDelta = TimeDelta::hours(48);

/// The end of the run log that is read: weeks of daily runs at about 12 KB a day.
const LOG_TAIL: u64 = 512 * 1024;

/// The job's settings, as far as the badge needs them.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Job {
    pub target: PathBuf,
    /// The sets that make one unit per child folder: the set's name, and the folder relative to
    /// the job's root (`Each`).
    pub each: Vec<(String, String)>,
    /// Paths relative to the job's root that it leaves out.
    pub exclude: Vec<String>,
}

/// Reads the job's settings file.
pub fn read_job(path: &Path) -> Result<Job, String> {
    let text = fs::read_to_string(path).map_err(|e| e.to_string())?;
    let value = psdata::parse(&text)?;
    let target = value
        .get("Target")
        .and_then(Value::as_str)
        .filter(|target| !target.trim().is_empty())
        .ok_or("it sets no Target")?;
    let each = value
        .get("Sets")
        .map(Value::items)
        .unwrap_or_default()
        .iter()
        .filter_map(|set| {
            let name = set.get("Name")?.as_str()?;
            let each = set.get("Each")?.as_str()?;
            Some((name.to_owned(), each.to_owned()))
        })
        .collect();
    let exclude = value
        .get("ExcludePaths")
        .map(Value::items)
        .unwrap_or_default()
        .iter()
        .filter_map(|path| path.as_str().map(str::to_owned))
        .collect();
    Ok(Job {
        target: PathBuf::from(target),
        each,
        exclude,
    })
}

/// What the run log says, from its tail.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Log {
    /// By unit, case aside.
    units: HashMap<String, UnitLog>,
    /// The newest run's end.
    pub last_run: Option<RunEnd>,
    /// The newest warning or error of a run as a whole, such as a target not ready.
    pub run_problem: Option<(DateTime<Utc>, String)>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct UnitLog {
    /// The newest `written` or `unchanged` event.
    checked: Option<DateTime<Utc>>,
    /// The newest warning or error for the unit.
    problem: Option<(DateTime<Utc>, String)>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunEnd {
    pub at: DateTime<Utc>,
    /// `ok`, `warn` or `error`.
    pub status: String,
}

/// Reads the log's tail.
pub fn read_log(path: &Path) -> Result<Log, String> {
    let mut file = File::open(path).map_err(|e| e.to_string())?;
    let length = file.metadata().map_err(|e| e.to_string())?.len();
    let start = length.saturating_sub(LOG_TAIL);
    file.seek(SeekFrom::Start(start))
        .map_err(|e| e.to_string())?;
    let mut bytes = Vec::new();
    file.read_to_end(&mut bytes).map_err(|e| e.to_string())?;
    let text = String::from_utf8_lossy(&bytes);
    // A tail starts inside a line, which is left out.
    let text = if start > 0 {
        text.split_once('\n').map_or("", |(_, rest)| rest)
    } else {
        &text
    };
    Ok(parse_log(text))
}

fn parse_log(text: &str) -> Log {
    let mut log = Log::default();
    let newer = |old: &Option<DateTime<Utc>>, at: DateTime<Utc>| old.is_none_or(|old| at >= old);
    for line in text.lines() {
        let Ok(entry) = serde_json::from_str::<serde_json::Value>(line) else {
            continue;
        };
        let Some(at) = entry["ts"].as_str().and_then(rfc3339::parse) else {
            continue;
        };
        let detail = entry["detail"].as_str().unwrap_or_default().to_owned();
        let unit = entry["unit"].as_str().map(str::to_lowercase);
        match (entry["event"].as_str().unwrap_or_default(), unit) {
            ("written" | "unchanged", Some(unit)) => {
                let unit = log.units.entry(unit).or_default();
                if newer(&unit.checked, at) {
                    unit.checked = Some(at);
                }
            }
            ("warn" | "error", Some(unit)) => {
                let unit = log.units.entry(unit).or_default();
                if newer(&unit.problem.as_ref().map(|p| p.0), at) {
                    unit.problem = Some((at, detail));
                }
            }
            ("warn" | "error", None) if newer(&log.run_problem.as_ref().map(|p| p.0), at) => {
                log.run_problem = Some((at, detail));
            }
            ("run-end", _) if newer(&log.last_run.as_ref().map(|r| r.at), at) => {
                log.last_run = Some(RunEnd {
                    at,
                    status: entry["status"].as_str().unwrap_or("?").to_owned(),
                });
            }
            _ => {}
        }
    }
    log
}

/// The job as one read found it: its settings, its log, and whether the target's drive is there.
#[derive(Debug, Clone)]
pub struct Reading {
    pub settings: PathBuf,
    pub job: Result<Job, String>,
    /// `None` when no log is set.
    pub log: Option<Result<Log, String>>,
    pub mounted: bool,
}

/// One project's backup.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Backup {
    /// The project's unit in the job; `None` when no set covers its folder.
    pub unit: Option<String>,
    /// The newest archive on the target; `None` when there is none or the drive is not there.
    pub archive: Option<Archive>,
    /// When the job last wrote the unit or found it unchanged: the log's word, else the newest
    /// archive's time.
    pub checked: Option<DateTime<Utc>>,
    /// A warning or error the job logged for the unit after its last check.
    pub problem: Option<String>,
    /// The project's paths the job leaves out, from its folder.
    pub excluded: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Archive {
    pub name: String,
    /// The time in its name, in the owner's time zone.
    pub at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum State {
    /// No set of the job covers the project's folder.
    NotCovered,
    /// Covered, but no archive and no log line says when it was last backed up.
    Unknown,
    Fresh,
    Stale,
    /// The job's last word on the unit is a warning or an error.
    Failing,
}

impl Backup {
    pub fn state(&self, now: DateTime<Utc>) -> State {
        match (&self.unit, &self.problem, self.checked) {
            (None, _, _) => State::NotCovered,
            (_, Some(_), _) => State::Failing,
            (_, None, None) => State::Unknown,
            (_, None, Some(at)) if now - at > STALE_AFTER => State::Stale,
            _ => State::Fresh,
        }
    }
}

impl Reading {
    /// Reads the settings, the log when one is set, and whether the target's drive is mounted.
    pub fn read(settings: &Path, log: Option<&Path>) -> Reading {
        let job = read_job(settings);
        let mounted = job
            .as_ref()
            .is_ok_and(|job| drive_root(&job.target).is_some_and(|root| root.exists()));
        Reading {
            settings: settings.to_owned(),
            job,
            log: log.map(read_log),
            mounted,
        }
    }

    pub fn last_run(&self) -> Option<&RunEnd> {
        self.log.as_ref()?.as_ref().ok()?.last_run.as_ref()
    }

    /// The backup of the project in `folder`, which sits in the data root.
    pub fn project(&self, folder: &Path) -> Backup {
        let Ok(job) = &self.job else {
            return Backup::default();
        };
        let Some(name) = folder.file_name().map(|n| n.to_string_lossy().into_owned()) else {
            return Backup::default();
        };
        let parent = folder.parent().unwrap_or(Path::new(""));
        let Some((set, each)) = job
            .each
            .iter()
            .find(|(_, each)| ends_with(parent, Path::new(each)))
        else {
            return Backup::default();
        };
        let unit = format!("{set}\\{name}");
        let archive = if self.mounted {
            newest_archive(&job.target.join(set).join(&name))
        } else {
            None
        };
        let logged = self
            .log
            .as_ref()
            .and_then(|log| log.as_ref().ok())
            .and_then(|log| log.units.get(&unit.to_lowercase()));
        let checked = [
            logged.and_then(|u| u.checked),
            archive.as_ref().and_then(|a| a.at),
        ]
        .into_iter()
        .flatten()
        .max();
        let problem = logged
            .and_then(|u| u.problem.as_ref())
            .filter(|(at, _)| checked.is_none_or(|checked| *at > checked))
            .map(|(_, detail)| detail.clone());
        let prefix = format!("{each}\\{name}\\")
            .replace('/', "\\")
            .to_lowercase();
        let excluded = job
            .exclude
            .iter()
            .filter_map(|path| {
                let path = path.replace('/', "\\");
                let rest = path.get(prefix.len()..)?;
                path.to_lowercase()
                    .starts_with(&prefix)
                    .then(|| rest.to_owned())
            })
            .collect();
        Backup {
            unit: Some(unit),
            archive,
            checked,
            problem,
            excluded,
        }
    }
}

/// `E:\Archive\…` → `E:\`.
fn drive_root(path: &Path) -> Option<PathBuf> {
    let mut components = path.components();
    match (components.next()?, components.next()?) {
        (prefix @ Component::Prefix(_), Component::RootDir) => {
            Some([prefix, Component::RootDir].iter().collect())
        }
        _ => None,
    }
}

/// Whether `path` ends with the components of `tail`, case aside.
fn ends_with(path: &Path, tail: &Path) -> bool {
    let mut path = path.components().rev();
    let tail: Vec<_> = tail.components().collect();
    !tail.is_empty()
        && tail.iter().rev().all(|t| {
            path.next().is_some_and(|p| {
                p.as_os_str()
                    .to_string_lossy()
                    .eq_ignore_ascii_case(&t.as_os_str().to_string_lossy())
            })
        })
}

/// The newest `.zip` in `dir` by name, which leads with its time.
fn newest_archive(dir: &Path) -> Option<Archive> {
    let name = fs::read_dir(dir)
        .ok()?
        .flatten()
        .filter(|entry| entry.file_type().is_ok_and(|t| t.is_file()))
        .filter_map(|entry| entry.file_name().into_string().ok())
        .filter(|name| name.to_lowercase().ends_with(".zip"))
        .max()?;
    Some(Archive {
        at: archive_time(&name),
        name,
    })
}

/// `2026-11-01 123001 alpha.zip` (or `… 1230 …`) → that time in the owner's time zone.
fn archive_time(name: &str) -> Option<DateTime<Utc>> {
    let (date, rest) = name.split_once(' ')?;
    let clock: String = rest.chars().take_while(char::is_ascii_digit).collect();
    let format = match clock.len() {
        6 => "%Y-%m-%d %H%M%S",
        4 => "%Y-%m-%d %H%M",
        _ => return None,
    };
    let naive = NaiveDateTime::parse_from_str(&format!("{date} {clock}"), format).ok()?;
    Local
        .from_local_datetime(&naive)
        .earliest()
        .map(|at| at.with_timezone(&Utc))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::files::tests::Scratch;

    const LOG: &str = r#"{"event":"run-start","detail":"target E:\\Archive","ts":"2026-10-30T12:30:01.1-04:00","run":"2026-10-30 123001"}
{"event":"written","unit":"WORK\\alpha","archive":"E:\\Archive\\WORK\\alpha\\2026-10-30 123001 alpha.zip","ts":"2026-10-30T12:30:05-04:00"}
{"event":"error","unit":"WORK\\beta","detail":"access denied","ts":"2026-10-30T12:30:06-04:00"}
{"event":"run-end","status":"error","written":1,"ts":"2026-10-30T12:30:09-04:00"}
not json at all
{"event":"run-start","ts":"2026-10-31T12:30:01-04:00"}
{"event":"unchanged","unit":"WORK\\Alpha","detail":"since 2026-10-30 123001 alpha.zip","ts":"2026-10-31T12:30:04-04:00"}
{"event":"warn","detail":"run skipped: E:\\ is not mounted","ts":"2026-10-31T12:30:05-04:00"}
{"event":"run-end","status":"warn","written":0,"ts":"2026-10-31T12:30:08-04:00"}
"#;

    fn at(text: &str) -> DateTime<Utc> {
        rfc3339::parse(text).unwrap()
    }

    #[test]
    fn reads_the_run_log() {
        let log = parse_log(LOG);
        assert_eq!(
            log.last_run,
            Some(RunEnd {
                at: at("2026-10-31T12:30:08-04:00"),
                status: "warn".to_owned()
            })
        );
        assert_eq!(
            log.run_problem.as_ref().map(|p| p.1.as_str()),
            Some(r"run skipped: E:\ is not mounted")
        );
        // Units compare case aside; the newest check wins.
        let alpha = &log.units["work\\alpha"];
        assert_eq!(alpha.checked, Some(at("2026-10-31T12:30:04-04:00")));
        assert_eq!(alpha.problem, None);
        let beta = &log.units["work\\beta"];
        assert_eq!(beta.checked, None);
        assert_eq!(beta.problem.as_ref().unwrap().1, "access denied");
    }

    #[test]
    fn reads_the_log_from_its_tail() {
        let scratch = Scratch::new("backup-log-tail");
        let path = scratch.0.join("log.jsonl");
        let filler = format!(
            "{{\"event\":\"note\",\"detail\":\"{}\"}}\n",
            "x".repeat(200)
        );
        let mut text = filler.repeat((LOG_TAIL / 200) as usize);
        text.push_str(LOG);
        fs::write(&path, text).unwrap();
        let log = read_log(&path).unwrap();
        assert_eq!(log.last_run.unwrap().status, "warn");
        assert!(read_log(&scratch.0.join("missing.jsonl")).is_err());
    }

    #[test]
    fn archive_names_carry_their_time() {
        let local = |text: &str| {
            let naive = NaiveDateTime::parse_from_str(text, "%Y-%m-%d %H:%M:%S").unwrap();
            Local
                .from_local_datetime(&naive)
                .earliest()
                .unwrap()
                .with_timezone(&Utc)
        };
        assert_eq!(
            archive_time("2026-11-01 123001 alpha.zip"),
            Some(local("2026-11-01 12:30:01"))
        );
        assert_eq!(
            archive_time("2026-10-28 0915 beta gamma.zip"),
            Some(local("2026-10-28 09:15:00"))
        );
        assert_eq!(archive_time("alpha.zip"), None);
        assert_eq!(archive_time("2026-11-01 12 alpha.zip"), None);
    }

    #[test]
    fn paths_compare_by_their_last_components() {
        assert!(ends_with(
            Path::new(r"C:\Home\Docs\WORK"),
            Path::new("work")
        ));
        assert!(ends_with(Path::new(r"C:\Home\Docs\a\b"), Path::new(r"A\B")));
        assert!(!ends_with(
            Path::new(r"C:\Home\Docs\WORK"),
            Path::new("other")
        ));
        assert!(!ends_with(Path::new(r"C:\x"), Path::new("")));
        assert_eq!(
            drive_root(Path::new(r"E:\Archive\Files")),
            Some(PathBuf::from(r"E:\"))
        );
        assert_eq!(drive_root(Path::new(r"relative\path")), None);
    }

    /// A job over a scratch tree: its settings, its log, and archives on a "target" that is a
    /// folder of the scratch's own drive.
    #[test]
    fn reads_each_projects_backup() {
        let scratch = Scratch::new("backup-reading");
        let root = &scratch.0;
        let target = root.join("target");
        let data = root.join("home").join("WORK");
        for name in ["alpha", "beta", "gamma"] {
            fs::create_dir_all(data.join(name)).unwrap();
        }
        let settings = root.join("job.psd1");
        fs::write(
            &settings,
            format!(
                "@{{\n    Target = '{}'\n    Sets = @(\n        @{{ Name = 'NOTES'; Paths = @('NOTES') }}\n        @{{ Name = 'WORK'; Each = 'WORK' }}\n    )\n    ExcludePaths = @(\n        'WORK\\alpha\\big data'\n        'WORK\\alphabet\\other'\n    )\n}}\n",
                target.display()
            ),
        )
        .unwrap();
        let alpha = target.join("WORK").join("alpha");
        fs::create_dir_all(&alpha).unwrap();
        for name in [
            "2026-10-29 123001 alpha.zip",
            "2026-10-30 123001 alpha.zip",
            "2026-10-30 123001 alpha.zip.partial",
        ] {
            fs::write(alpha.join(name), "").unwrap();
        }
        let log = root.join("log.jsonl");
        fs::write(&log, LOG).unwrap();

        let reading = Reading::read(&settings, Some(&log));
        assert!(reading.mounted);
        assert_eq!(reading.last_run().unwrap().status, "warn");
        let job = reading.job.as_ref().unwrap();
        assert_eq!(job.each, [("WORK".to_owned(), "WORK".to_owned())]);

        let backup = reading.project(&data.join("alpha"));
        assert_eq!(backup.unit.as_deref(), Some(r"WORK\alpha"));
        assert_eq!(
            backup.archive.as_ref().map(|a| a.name.as_str()),
            Some("2026-10-30 123001 alpha.zip")
        );
        // The log's newer word (unchanged on the 31st) is the last check.
        assert_eq!(backup.checked, Some(at("2026-10-31T12:30:04-04:00")));
        assert_eq!(backup.excluded, ["big data"]);
        assert_eq!(backup.state(at("2026-11-01T12:00:00-04:00")), State::Fresh);
        assert_eq!(backup.state(at("2026-11-03T12:31:00-04:00")), State::Stale);
        // An error after the last check; no archive and no check at all.
        let beta = reading.project(&data.join("beta"));
        assert_eq!(beta.problem.as_deref(), Some("access denied"));
        assert_eq!(beta.state(Utc::now()), State::Failing);
        assert_eq!(
            reading.project(&data.join("gamma")).state(Utc::now()),
            State::Unknown
        );
        // A folder outside every set is not covered.
        let elsewhere = reading.project(&root.join("other").join("delta"));
        assert_eq!(elsewhere, Backup::default());
        assert_eq!(elsewhere.state(Utc::now()), State::NotCovered);

        // Settings that cannot be read leave every badge empty.
        let broken = Reading::read(&root.join("missing.psd1"), None);
        assert!(broken.job.is_err() && !broken.mounted);
        assert_eq!(broken.project(&data.join("alpha")), Backup::default());
        fs::write(&settings, "@{ Sets = @() }").unwrap();
        assert!(read_job(&settings).unwrap_err().contains("Target"));
    }
}
