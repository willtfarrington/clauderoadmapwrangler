//! Stack chips (D-25): what a project is built with, read from its manifests at the root and one
//! folder down (a nested package, a `scripts` folder), with no child. `Cargo.toml` → Rust and the
//! GUI family it depends on; `pyproject.toml` → Python with its version pin and uv or Poetry;
//! `package.json` → TypeScript or Node with the package manager; a compose file or a `Dockerfile`
//! → Docker; a `.ps1` script → PowerShell; `index.html` at the root → a static site.

use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};

/// Folders never looked into: build output, dependencies, environments, data and the roadmap.
const SKIP: [&str; 14] = [
    "target",
    "node_modules",
    "dist",
    "build",
    "venv",
    "data",
    "private",
    "roadmap",
    "docs",
    "fixtures",
    "vendor",
    "out",
    "bin",
    "obj",
];

/// The GUI crates a `Cargo.toml` may name, and the chip's word for each.
const GUI: [(&str, &str); 7] = [
    ("eframe", "egui"),
    ("egui", "egui"),
    ("iced", "iced"),
    ("tauri", "Tauri"),
    ("slint", "Slint"),
    ("dioxus", "Dioxus"),
    ("gtk4", "GTK"),
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Kind {
    Rust,
    Python,
    TypeScript,
    Node,
    PowerShell,
    Docker,
    Static,
}

impl Kind {
    pub const ALL: [Kind; 7] = [
        Kind::Rust,
        Kind::Python,
        Kind::TypeScript,
        Kind::Node,
        Kind::PowerShell,
        Kind::Docker,
        Kind::Static,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Kind::Rust => "Rust",
            Kind::Python => "Python",
            Kind::TypeScript => "TypeScript",
            Kind::Node => "Node",
            Kind::PowerShell => "PowerShell",
            Kind::Docker => "Docker",
            Kind::Static => "Static site",
        }
    }

    /// The chip's text in the compact row.
    pub fn short(self) -> &'static str {
        match self {
            Kind::Rust => "RS",
            Kind::Python => "PY",
            Kind::TypeScript => "TS",
            Kind::Node => "JS",
            Kind::PowerShell => "PS",
            Kind::Docker => "DKR",
            Kind::Static => "WEB",
        }
    }
}

/// One chip: the stack, a detail (a version pin, a manager, the GUI family) and where it was
/// found, from the project folder.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Chip {
    pub kind: Kind,
    pub detail: Option<String>,
    pub source: String,
}

impl Chip {
    /// `Python 3.13 · uv`, `Rust · egui`, `PowerShell`.
    pub fn text(&self) -> String {
        match &self.detail {
            Some(detail) => format!("{} · {detail}", self.kind.label()),
            None => self.kind.label().to_owned(),
        }
    }
}

/// The chips of the project at `folder`, one per stack, in [`Kind`] order; the root's manifest
/// wins over a nested one.
pub fn read(folder: &Path) -> Vec<Chip> {
    let mut places = vec![(folder.to_owned(), String::new())];
    if let Ok(entries) = fs::read_dir(folder) {
        let mut nested: Vec<_> = entries
            .flatten()
            .filter(|entry| entry.file_type().is_ok_and(|t| t.is_dir()))
            .filter_map(|entry| {
                let name = entry.file_name().into_string().ok()?;
                let skipped = name.starts_with(['.', '_'])
                    || SKIP.iter().any(|skip| skip.eq_ignore_ascii_case(&name));
                (!skipped).then(|| (entry.path(), format!("{name}/")))
            })
            .collect();
        nested.sort_by(|a, b| a.1.cmp(&b.1));
        places.extend(nested);
    }
    let mut chips: Vec<Chip> = Vec::new();
    for (dir, prefix) in &places {
        let root = prefix.is_empty();
        for chip in chips_in(dir, prefix, root) {
            if !chips.iter().any(|c| c.kind == chip.kind) {
                chips.push(chip);
            }
        }
    }
    chips.sort_by_key(|chip| chip.kind);
    chips
}

/// The chips one folder's own files give.
fn chips_in(dir: &Path, prefix: &str, root: bool) -> Vec<Chip> {
    let has = |name: &str| dir.join(name).is_file();
    let text = |name: &str| fs::read_to_string(dir.join(name)).ok();
    let source = |name: &str| format!("{prefix}{name}");
    let mut chips = Vec::new();
    if let Some(manifest) = text("Cargo.toml") {
        chips.push(Chip {
            kind: Kind::Rust,
            detail: gui_family(&manifest).map(str::to_owned),
            source: source("Cargo.toml"),
        });
    }
    if let Some(manifest) = text("pyproject.toml") {
        let pin = text(".python-version")
            .and_then(|t| t.lines().next().map(|l| l.trim().to_owned()))
            .filter(|pin| !pin.is_empty())
            .or_else(|| requires_python(&manifest));
        let manager = if has("uv.lock") {
            Some("uv")
        } else if has("poetry.lock") {
            Some("Poetry")
        } else {
            None
        };
        chips.push(Chip {
            kind: Kind::Python,
            detail: joined(pin.as_deref(), manager),
            source: source("pyproject.toml"),
        });
    }
    if let Some(manifest) = text("package.json") {
        let json: serde_json::Value = serde_json::from_str(&manifest).unwrap_or_default();
        let typed = has("tsconfig.json")
            || ["dependencies", "devDependencies"]
                .iter()
                .any(|deps| json[deps].get("typescript").is_some());
        let manager = json["packageManager"]
            .as_str()
            .and_then(|spec| spec.split('@').next())
            .filter(|name| !name.is_empty())
            .map(str::to_owned)
            .or_else(|| {
                [
                    ("pnpm-lock.yaml", "pnpm"),
                    ("yarn.lock", "yarn"),
                    ("bun.lock", "bun"),
                    ("bun.lockb", "bun"),
                    ("package-lock.json", "npm"),
                ]
                .iter()
                .find(|(lock, _)| has(lock))
                .map(|(_, name)| (*name).to_owned())
            });
        chips.push(Chip {
            kind: if typed { Kind::TypeScript } else { Kind::Node },
            detail: manager,
            source: source("package.json"),
        });
    }
    let compose = [
        "docker-compose.yml",
        "docker-compose.yaml",
        "compose.yml",
        "compose.yaml",
    ]
    .into_iter()
    .find(|name| has(name));
    if let Some(name) = compose.or_else(|| has("Dockerfile").then_some("Dockerfile")) {
        chips.push(Chip {
            kind: Kind::Docker,
            detail: compose.map(|_| "compose".to_owned()),
            source: source(name),
        });
    }
    if let Some(script) = first_with_extension(dir, "ps1") {
        chips.push(Chip {
            kind: Kind::PowerShell,
            detail: None,
            source: source(&script),
        });
    }
    if root && has("index.html") {
        chips.push(Chip {
            kind: Kind::Static,
            detail: None,
            source: source("index.html"),
        });
    }
    chips
}

/// The GUI family a `Cargo.toml` depends on: a dependency key (`eframe = …`, `eframe.workspace =
/// true`) or a `[dependencies.eframe]` table.
fn gui_family(manifest: &str) -> Option<&'static str> {
    manifest.lines().map(str::trim).find_map(|line| {
        let key = line
            .strip_prefix('[')
            .and_then(|table| table.strip_suffix(']'))
            .and_then(|table| table.rsplit('.').next())
            .unwrap_or(line);
        let key = key.split(['=', '.', ' ']).next().unwrap_or_default().trim();
        GUI.iter()
            .find(|(name, _)| key.eq_ignore_ascii_case(name))
            .map(|(_, family)| *family)
    })
}

/// `requires-python = ">=3.13"` → `>=3.13`.
fn requires_python(manifest: &str) -> Option<String> {
    manifest.lines().find_map(|line| {
        let (key, value) = line.split_once('=')?;
        (key.trim() == "requires-python").then(|| value.trim().trim_matches(['"', '\'']).to_owned())
    })
}

fn joined(a: Option<&str>, b: Option<&str>) -> Option<String> {
    match (a, b) {
        (Some(a), Some(b)) => Some(format!("{a} · {b}")),
        (Some(one), None) | (None, Some(one)) => Some(one.to_owned()),
        (None, None) => None,
    }
}

/// The first file in `dir` with the extension, by name.
fn first_with_extension(dir: &Path, extension: &str) -> Option<String> {
    let mut names: Vec<String> = fs::read_dir(dir)
        .ok()?
        .flatten()
        .filter(|entry| entry.file_type().is_ok_and(|t| t.is_file()))
        .filter_map(|entry| entry.file_name().into_string().ok())
        .filter(|name| {
            Path::new(name)
                .extension()
                .is_some_and(|ext| ext.eq_ignore_ascii_case(extension))
        })
        .collect();
    names.sort();
    names.into_iter().next()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::files::tests::Scratch;

    fn write(root: &Path, relative: &str, text: &str) {
        let path = root.join(relative);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, text).unwrap();
    }

    fn texts(chips: &[Chip]) -> Vec<String> {
        chips.iter().map(Chip::text).collect()
    }

    #[test]
    fn reads_the_manifests_at_the_root_and_one_folder_down() {
        let scratch = Scratch::new("stack-mixed");
        let root = &scratch.0;
        write(
            root,
            "Cargo.toml",
            "[workspace]\nmembers = [\"crates/*\"]\n\n[workspace.dependencies]\neframe = { version = \"=0.37.0\" }\n",
        );
        write(root, "scripts/install.ps1", "Write-Host hi");
        write(
            root,
            "service/pyproject.toml",
            "[project]\nname = \"service\"\nrequires-python = \">=3.13\"\n",
        );
        write(root, "service/uv.lock", "");
        write(
            root,
            "web/package.json",
            r#"{"packageManager": "pnpm@9.1.0"}"#,
        );
        write(root, "web/tsconfig.json", "{}");
        write(root, "compose.yaml", "services: {}");
        write(root, "index.html", "<!doctype html>");
        // Nothing is read from the folders left out.
        write(root, "target/debug/package.json", "{}");
        write(root, "node_modules/x/Cargo.toml", "");
        write(root, ".venv/pyvenv.cfg", "");
        let chips = read(root);
        assert_eq!(
            texts(&chips),
            [
                "Rust · egui",
                "Python · >=3.13 · uv",
                "TypeScript · pnpm",
                "PowerShell",
                "Docker · compose",
                "Static site",
            ]
        );
        assert_eq!(chips[1].source, "service/pyproject.toml");
        assert_eq!(chips[3].source, "scripts/install.ps1");
    }

    #[test]
    fn the_details_have_their_fallbacks() {
        let scratch = Scratch::new("stack-details");
        let root = &scratch.0;
        write(
            root,
            "Cargo.toml",
            "[package]\nname = \"x\"\n\n[dependencies.iced]\nversion = \"0.14\"\n",
        );
        write(root, "pyproject.toml", "[project]\nname = \"x\"\n");
        write(root, ".python-version", "3.12\n");
        write(
            root,
            "package.json",
            r#"{"devDependencies": {"jest": "1"}}"#,
        );
        write(root, "package-lock.json", "{}");
        write(root, "Dockerfile", "FROM scratch");
        // A nested index.html is not a site; a nested manifest of a stack found already loses.
        write(root, "site/index.html", "");
        write(
            root,
            "tool/Cargo.toml",
            "[dependencies]\neframe.workspace = true\n",
        );
        assert_eq!(
            texts(&read(root)),
            ["Rust · iced", "Python · 3.12", "Node · npm", "Docker"]
        );
        let empty = Scratch::new("stack-empty");
        assert!(read(&empty.0).is_empty());
        assert!(read(&empty.0.join("missing")).is_empty());
    }

    #[test]
    fn finds_the_gui_family_by_its_dependency_key() {
        assert_eq!(
            gui_family("[dependencies]\neframe = \"0.37\""),
            Some("egui")
        );
        assert_eq!(gui_family("egui.workspace = true"), Some("egui"));
        assert_eq!(
            gui_family("[dependencies.tauri]\nversion = \"2\""),
            Some("Tauri")
        );
        assert_eq!(
            gui_family("[dependencies]\nserde = \"1\"\n# eframe later"),
            None
        );
        assert_eq!(gui_family("egui_plot = \"0.38\""), None);
    }
}
