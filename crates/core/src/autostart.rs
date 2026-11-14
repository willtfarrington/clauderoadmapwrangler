//! The start-at-login entry (D-29; EP-28a): the value `Roadmap Wrangler` under the Run key of the
//! owner's Windows user, `HKCU\Software\Microsoft\Windows\CurrentVersion\Run`, holding the
//! installed window's path with [`FLAG`], so that Windows starts the window hidden in the tray at
//! each login and the ledger hears every session. It is written and removed only on the owner's
//! word: `crw autostart on|off`, or the window's File > Start at login. A trial (`CRW_HOME`) and
//! the tests keep the entry in a file of their own, `autostart.txt` in the home, which holds the
//! command as the value would, so that neither ever writes the owner's registry (rule 13).

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use crate::{paths, process};

/// The Run value's name.
pub const NAME: &str = "Roadmap Wrangler";
/// The window's flag: start hidden in the tray.
pub const FLAG: &str = "--autostart";
/// The Run key under `HKEY_CURRENT_USER`.
pub const RUN_KEY: &str = r"Software\Microsoft\Windows\CurrentVersion\Run";
/// The file that stands in for the value in a trial.
pub const SCRATCH_FILE: &str = "autostart.txt";

/// Where the entry lives.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Place {
    /// The Run key of the owner's Windows user.
    Run,
    /// A trial's or a test's file, which holds the command as the value would.
    File(PathBuf),
}

impl Place {
    /// `CRW_HOME`'s `autostart.txt` in a trial, else the Run key.
    pub fn current() -> Place {
        match paths::home() {
            Some(home) => Place::File(home.join(SCRATCH_FILE)),
            None => Place::Run,
        }
    }

    /// Where the entry lives, in words.
    pub fn words(&self) -> String {
        match self {
            Place::Run => {
                format!("the Run key of your Windows user (HKCU\\{RUN_KEY}, value {NAME})")
            }
            Place::File(path) => format!("the trial's file {}", path.display()),
        }
    }
}

/// The entry's command for the window at `exe`: `"<exe>" --autostart`.
pub fn command(exe: &Path) -> String {
    format!("\"{}\" {FLAG}", exe.display())
}

/// The program a command starts: the quoted path, or the first word.
pub fn exe_of(command: &str) -> Option<PathBuf> {
    let command = command.trim();
    let program = match command.strip_prefix('"') {
        Some(rest) => rest.split('"').next().unwrap_or_default(),
        None => command.split_whitespace().next().unwrap_or_default(),
    };
    (!program.trim().is_empty()).then(|| PathBuf::from(program))
}

/// The entry's command, or none when there is no entry.
pub fn read(place: &Place) -> Result<Option<String>, String> {
    match place {
        Place::Run => run::read(),
        Place::File(path) => match fs::read_to_string(path) {
            Ok(text) => Ok(Some(text.trim().to_owned())),
            Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(None),
            Err(e) => Err(format!("{}: {e}", path.display())),
        },
    }
}

/// Writes the entry: [`command`] for the window at `exe`, in place of any there. A trial's file
/// is made with its folder.
pub fn register(place: &Place, exe: &Path) -> Result<(), String> {
    let command = command(exe);
    match place {
        Place::Run => run::write(&command),
        Place::File(path) => {
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent).map_err(|e| format!("{}: {e}", parent.display()))?;
            }
            fs::write(path, command).map_err(|e| format!("{}: {e}", path.display()))
        }
    }
}

/// Removes the entry: whether there was one.
pub fn remove(place: &Place) -> Result<bool, String> {
    match place {
        Place::Run => run::delete(),
        Place::File(path) => match fs::remove_file(path) {
            Ok(()) => Ok(true),
            Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(false),
            Err(e) => Err(format!("{}: {e}", path.display())),
        },
    }
}

/// The entry as found.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Status {
    Absent,
    /// The entry's command; the program it starts, none when the command names none; whether
    /// that is the installed window, and whether its file is there.
    Starts {
        command: String,
        exe: Option<PathBuf>,
        installed: bool,
        present: bool,
    },
}

/// The entry at `place`, set against the installed window at `installed_window`: `installed` when
/// the program it starts is that exe, `present` when the program's file is there.
pub fn status(place: &Place, installed_window: Option<&Path>) -> Result<Status, String> {
    let Some(command) = read(place)? else {
        return Ok(Status::Absent);
    };
    let exe = exe_of(&command);
    let installed = exe
        .as_deref()
        .zip(installed_window)
        .is_some_and(|(exe, window)| paths::same(exe, window));
    let present = exe.as_deref().is_some_and(Path::is_file);
    Ok(Status::Starts {
        command,
        exe,
        installed,
        present,
    })
}

impl Status {
    pub fn is_registered(&self) -> bool {
        matches!(self, Status::Starts { .. })
    }

    /// The entry in words.
    pub fn words(&self) -> String {
        match self {
            Status::Absent => "not registered: the window does not start at login".to_owned(),
            Status::Starts {
                exe: None, command, ..
            } => {
                format!("registered, but its command names no program: {command}")
            }
            Status::Starts {
                installed: true,
                present: true,
                ..
            } => "starts the installed window hidden in the tray at login".to_owned(),
            Status::Starts {
                exe: Some(exe),
                present: false,
                ..
            } => format!(
                "starts {} at login, which is not there: {}",
                exe.display(),
                process::GONE
            ),
            Status::Starts { exe: Some(exe), .. } => {
                format!(
                    "starts {} at login, not the installed window",
                    exe.display()
                )
            }
        }
    }
}

/// Whether the window was started at login: an argument equal to [`FLAG`].
pub fn started_at_login<S: AsRef<str>>(args: &[S]) -> bool {
    args.iter().any(|arg| arg.as_ref() == FLAG)
}

/// The Run value, through the registry functions of the pinned `windows` crate.
#[cfg(windows)]
mod run {
    use windows::Win32::Foundation::{ERROR_FILE_NOT_FOUND, ERROR_MORE_DATA, WIN32_ERROR};
    use windows::Win32::System::Registry::{
        HKEY_CURRENT_USER, REG_SZ, RRF_RT_REG_SZ, RegDeleteKeyValueW, RegGetValueW, RegSetKeyValueW,
    };
    use windows::core::PCWSTR;

    use super::{NAME, RUN_KEY};

    /// `text` as a NUL-terminated wide string.
    fn wide(text: &str) -> Vec<u16> {
        text.encode_utf16().chain(std::iter::once(0)).collect()
    }

    fn failed(what: &str, status: WIN32_ERROR) -> String {
        match status.ok() {
            Ok(()) => format!("the Run key's value was not {what}"),
            Err(e) => format!("the Run key's value was not {what}: {}", e.message()),
        }
    }

    /// The value, or none when it is not there.
    pub(super) fn read() -> Result<Option<String>, String> {
        let (key, name) = (wide(RUN_KEY), wide(NAME));
        // Its size first, then the value; a value that grew between the two is asked again.
        for _ in 0..3 {
            let mut bytes = 0u32;
            // SAFETY: the key and the name are NUL-terminated wide strings that outlive the call;
            // no buffer is passed, so only the size is written, into a live u32.
            let status = unsafe {
                RegGetValueW(
                    HKEY_CURRENT_USER,
                    PCWSTR(key.as_ptr()),
                    PCWSTR(name.as_ptr()),
                    RRF_RT_REG_SZ,
                    None,
                    None,
                    Some(&raw mut bytes),
                )
            };
            if status == ERROR_FILE_NOT_FOUND {
                return Ok(None);
            }
            if status.is_err() {
                return Err(failed("read", status));
            }
            let mut buffer = vec![0u16; (bytes as usize).div_ceil(2).max(1)];
            let mut size = (buffer.len() * 2) as u32;
            // SAFETY: as above, and the buffer is passed with its size in bytes, which the call
            // writes back.
            let status = unsafe {
                RegGetValueW(
                    HKEY_CURRENT_USER,
                    PCWSTR(key.as_ptr()),
                    PCWSTR(name.as_ptr()),
                    RRF_RT_REG_SZ,
                    None,
                    Some(buffer.as_mut_ptr().cast()),
                    Some(&raw mut size),
                )
            };
            if status == ERROR_MORE_DATA {
                continue;
            }
            if status == ERROR_FILE_NOT_FOUND {
                return Ok(None);
            }
            if status.is_err() {
                return Err(failed("read", status));
            }
            let written = &buffer[..(size as usize / 2).min(buffer.len())];
            let end = written
                .iter()
                .position(|c| *c == 0)
                .unwrap_or(written.len());
            return Ok(Some(String::from_utf16_lossy(&written[..end])));
        }
        Err("the Run key's value kept changing while it was read".to_owned())
    }

    /// Writes `command` as the value, a string.
    pub(super) fn write(command: &str) -> Result<(), String> {
        let (key, name, data) = (wide(RUN_KEY), wide(NAME), wide(command));
        let bytes = u32::try_from(data.len() * 2).map_err(|_| "the command is too long")?;
        // SAFETY: the key, the name and the data are NUL-terminated wide strings that outlive the
        // call, and the data is passed with its size in bytes, its terminator included.
        let status = unsafe {
            RegSetKeyValueW(
                HKEY_CURRENT_USER,
                PCWSTR(key.as_ptr()),
                PCWSTR(name.as_ptr()),
                REG_SZ.0,
                Some(data.as_ptr().cast()),
                bytes,
            )
        };
        if status.is_err() {
            return Err(failed("written", status));
        }
        Ok(())
    }

    /// Deletes the value: whether it was there.
    pub(super) fn delete() -> Result<bool, String> {
        let (key, name) = (wide(RUN_KEY), wide(NAME));
        // SAFETY: the key and the name are NUL-terminated wide strings that outlive the call.
        let status = unsafe {
            RegDeleteKeyValueW(
                HKEY_CURRENT_USER,
                PCWSTR(key.as_ptr()),
                PCWSTR(name.as_ptr()),
            )
        };
        if status == ERROR_FILE_NOT_FOUND {
            return Ok(false);
        }
        if status.is_err() {
            return Err(failed("deleted", status));
        }
        Ok(true)
    }
}

#[cfg(not(windows))]
mod run {
    const ONLY: &str = "only Windows has a Run key";

    pub(super) fn read() -> Result<Option<String>, String> {
        Err(ONLY.to_owned())
    }

    pub(super) fn write(_command: &str) -> Result<(), String> {
        Err(ONLY.to_owned())
    }

    pub(super) fn delete() -> Result<bool, String> {
        Err(ONLY.to_owned())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::files::tests::Scratch;

    #[test]
    fn the_command_quotes_the_window_and_its_program_reads_back() {
        let exe = Path::new(r"C:\Some Folder\bin\clauderoadmapwrangler.exe");
        let command = command(exe);
        assert_eq!(
            command,
            r#""C:\Some Folder\bin\clauderoadmapwrangler.exe" --autostart"#
        );
        assert_eq!(exe_of(&command).as_deref(), Some(exe));
        // A bare first word, as a hand-written entry may be.
        assert_eq!(
            exe_of(r"C:\tools\window.exe --autostart").as_deref(),
            Some(Path::new(r"C:\tools\window.exe"))
        );
        assert_eq!(
            exe_of("  window.exe").as_deref(),
            Some(Path::new("window.exe"))
        );
        assert_eq!(exe_of(""), None);
        assert_eq!(exe_of("\"\" --autostart"), None);
    }

    #[test]
    fn the_entry_is_registered_read_and_removed_in_a_trials_file() {
        let scratch = Scratch::new("autostart-file");
        let place = Place::File(scratch.0.join("home").join(SCRATCH_FILE));
        assert_eq!(read(&place).unwrap(), None);
        assert_eq!(status(&place, None).unwrap(), Status::Absent);
        assert!(!remove(&place).unwrap());
        let window = scratch.0.join("bin").join("clauderoadmapwrangler.exe");
        // The folder is made with the file.
        register(&place, &window).unwrap();
        assert_eq!(read(&place).unwrap(), Some(command(&window)));
        assert!(status(&place, None).unwrap().is_registered());
        // Registered again: the one entry, in place of the other.
        let other = scratch.0.join("elsewhere.exe");
        register(&place, &other).unwrap();
        assert_eq!(read(&place).unwrap(), Some(command(&other)));
        assert!(remove(&place).unwrap());
        assert_eq!(read(&place).unwrap(), None);
        assert!(!remove(&place).unwrap());
        assert!(place.words().starts_with("the trial's file "));
        assert!(
            Place::Run
                .words()
                .contains(r"HKCU\Software\Microsoft\Windows")
        );
    }

    #[test]
    fn the_status_says_whether_it_starts_the_installed_window() {
        let scratch = Scratch::new("autostart-status");
        let place = Place::File(scratch.0.join(SCRATCH_FILE));
        let bin = scratch.0.join("bin");
        fs::create_dir_all(&bin).unwrap();
        let window = bin.join("clauderoadmapwrangler.exe");
        fs::write(&window, "window").unwrap();
        let installed = Some(window.as_path());

        register(&place, &window).unwrap();
        let found = status(&place, installed).unwrap();
        assert_eq!(
            found,
            Status::Starts {
                command: command(&window),
                exe: Some(window.clone()),
                installed: true,
                present: true,
            }
        );
        assert_eq!(
            found.words(),
            "starts the installed window hidden in the tray at login"
        );
        // The same path in another case is the same exe, as Windows compares them.
        let upper = PathBuf::from(window.display().to_string().to_uppercase());
        assert!(matches!(
            status(&place, Some(&upper)).unwrap(),
            Status::Starts {
                installed: true,
                ..
            }
        ));

        // Another window that is there.
        let elsewhere = scratch.0.join("target").join("clauderoadmapwrangler.exe");
        fs::create_dir_all(elsewhere.parent().unwrap()).unwrap();
        fs::write(&elsewhere, "build").unwrap();
        register(&place, &elsewhere).unwrap();
        let found = status(&place, installed).unwrap();
        assert!(matches!(
            found,
            Status::Starts {
                installed: false,
                present: true,
                ..
            }
        ));
        assert!(found.words().ends_with("not the installed window"));

        // A window that is gone, the installed one too.
        fs::remove_file(&window).unwrap();
        register(&place, &window).unwrap();
        let found = status(&place, installed).unwrap();
        assert!(matches!(
            found,
            Status::Starts {
                installed: true,
                present: false,
                ..
            }
        ));
        assert!(found.words().contains(process::GONE), "{}", found.words());

        // A command that names no program.
        fs::write(scratch.0.join(SCRATCH_FILE), "\"\"").unwrap();
        let found = status(&place, installed).unwrap();
        assert!(matches!(found, Status::Starts { exe: None, .. }));
        assert!(found.words().contains("names no program"));
    }

    #[test]
    fn the_flag_says_a_start_at_login() {
        assert!(started_at_login(&["--autostart"]));
        assert!(started_at_login(&["x", "--autostart"]));
        assert!(!started_at_login(&["--restarted", "42"]));
        assert!(!started_at_login(&["--autostarted"]));
        assert!(!started_at_login::<&str>(&[]));
    }
}
