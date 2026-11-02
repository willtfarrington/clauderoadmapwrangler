//! The models Claude Code offers (D-22): the catalog it caches in
//! `%USERPROFILE%\.claude\cache\model-catalog\`, one file per surface, read for the launch dialog's
//! model and effort pickers, with a built-in list when no file reads. The files are only read;
//! their names carry account identifiers, which nothing here keeps or shows. A file the format
//! does not fit is passed over, so a change on Claude Code's side leaves the built-in list, not an
//! error.

use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use chrono::{DateTime, Utc};
use serde_json::Value;

use crate::rules::EFFORTS;
use crate::version::Version;

/// The catalog format this build reads.
const FORMAT: u64 = 2;

/// The CLI's surface, preferred; the desktop's and the extension's (`ccd`) list the same models.
const CLI_SURFACE: &str = "cc";

/// The models of the built-in list, the main section's in Claude Code's order: id and name.
const BUILT_IN: [(&str, &str, &str); 4] = [
    ("claude-opus-5-5", "Opus 5.5", "opus"),
    ("claude-fable-5-1", "Fable 5.1", "fable"),
    ("claude-sonnet-5-5", "Sonnet 5.5", "sonnet"),
    ("claude-haiku-5-5", "Haiku 5.5", "haiku"),
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Model {
    pub id: String,
    pub name: String,
    /// In the catalog's main section: the models Claude Code offers first.
    pub main: bool,
    /// The effort levels it takes, in order; empty when the catalog does not say.
    pub efforts: Vec<String>,
    pub default_effort: Option<String>,
    /// The oldest CLI that runs it, when the catalog says.
    pub min_cli: Option<Version>,
}

/// Where a catalog came from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Source {
    /// Claude Code's cache: the surface and when Claude Code fetched it.
    Cached {
        surface: String,
        fetched: Option<DateTime<Utc>>,
    },
    /// No cached file could be read, for this reason.
    BuiltIn(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Catalog {
    /// The main section first, each section in the catalog's order.
    pub models: Vec<Model>,
    /// Each alias and the model it stands for: `fable` and `claude-fable-5-1`.
    pub aliases: Vec<(String, String)>,
    pub source: Source,
}

impl Catalog {
    /// The folder Claude Code caches the catalog in.
    pub fn folder() -> Option<PathBuf> {
        env::var_os("USERPROFILE")
            .map(|home| PathBuf::from(home).join(r".claude\cache\model-catalog"))
    }

    /// The best file of `folder`: the CLI's surface first, then the latest fetched; the built-in
    /// list when none reads.
    pub fn read(folder: &Path) -> Catalog {
        let entries = match fs::read_dir(folder) {
            Ok(entries) => entries,
            Err(e) => return Catalog::built_in(format!("no catalog folder ({e})")),
        };
        let mut best: Option<Catalog> = None;
        let mut first_problem = None;
        for path in entries.filter_map(Result::ok).map(|entry| entry.path()) {
            if path
                .extension()
                .is_none_or(|ext| !ext.eq_ignore_ascii_case("json"))
            {
                continue;
            }
            let read = fs::read_to_string(&path)
                .map_err(|e| e.to_string())
                .and_then(|text| Catalog::parse(&text));
            match read {
                Ok(catalog) if best.as_ref().is_none_or(|b| catalog.rank() > b.rank()) => {
                    best = Some(catalog);
                }
                Ok(_) => {}
                Err(e) => {
                    first_problem.get_or_insert(e);
                }
            }
        }
        best.unwrap_or_else(|| {
            Catalog::built_in(match first_problem {
                Some(problem) => format!("no cached catalog could be read ({problem})"),
                None => "no cached catalog".to_owned(),
            })
        })
    }

    /// One cached file's text, in format 2: `catalog.config.models[]` with each model's section,
    /// effort levels and minimum CLI version, and `provider_alias_targets`.
    pub fn parse(text: &str) -> Result<Catalog, String> {
        let json: Value = serde_json::from_str(text).map_err(|e| format!("not JSON: {e}"))?;
        match json["version"].as_u64() {
            Some(FORMAT) => {}
            other => {
                return Err(format!(
                    "format {} (this build reads {FORMAT})",
                    other.map_or("unknown".to_owned(), |v| v.to_string())
                ));
            }
        }
        let catalog = &json["catalog"];
        let config = &catalog["config"];
        let strings = |value: &Value| -> Vec<String> {
            value
                .as_array()
                .map(|items| {
                    items
                        .iter()
                        .filter_map(Value::as_str)
                        .map(str::to_owned)
                        .collect()
                })
                .unwrap_or_default()
        };
        let mut models: Vec<Model> = config["models"]
            .as_array()
            .ok_or("no models list")?
            .iter()
            .filter_map(|model| {
                let id = model["id"].as_str()?.trim();
                let runtime = &model["runtime"];
                (!id.is_empty()).then(|| Model {
                    id: id.to_owned(),
                    name: model["name"].as_str().unwrap_or(id).to_owned(),
                    main: model["section"].as_str() == Some("main"),
                    efforts: strings(&runtime["effort_levels"]),
                    default_effort: runtime["default_effort"].as_str().map(str::to_owned),
                    min_cli: model["min_claude_code_version"]
                        .as_str()
                        .and_then(|v| v.trim().parse().ok()),
                })
            })
            .collect();
        if models.is_empty() {
            return Err("it lists no model".to_owned());
        }
        // A stable sort: the main section first, each in the catalog's order.
        models.sort_by_key(|model| !model.main);
        let aliases = config["provider_alias_targets"]
            .as_object()
            .map(|targets| {
                targets
                    .iter()
                    .filter_map(|(alias, target)| {
                        Some((alias.clone(), target["default"].as_str()?.to_owned()))
                    })
                    .collect()
            })
            .unwrap_or_default();
        Ok(Catalog {
            models,
            aliases,
            source: Source::Cached {
                surface: catalog["surface"].as_str().unwrap_or_default().to_owned(),
                fetched: json["fetchedAt"]
                    .as_i64()
                    .and_then(DateTime::from_timestamp_millis),
            },
        })
    }

    /// The models of the main section as Claude Code 2.1.316 listed them, every effort level each.
    pub fn built_in(why: String) -> Catalog {
        Catalog {
            models: BUILT_IN
                .iter()
                .map(|&(id, name, _)| Model {
                    id: id.to_owned(),
                    name: name.to_owned(),
                    main: true,
                    efforts: EFFORTS.map(str::to_owned).to_vec(),
                    default_effort: None,
                    min_cli: None,
                })
                .collect(),
            aliases: BUILT_IN
                .iter()
                .map(|&(id, _, alias)| (alias.to_owned(), id.to_owned()))
                .collect(),
            source: Source::BuiltIn(why),
        }
    }

    /// The model `name` names: an id, or an alias resolved, case aside.
    pub fn model(&self, name: &str) -> Option<&Model> {
        let id = self.resolve(name);
        self.models
            .iter()
            .find(|model| model.id.eq_ignore_ascii_case(id))
    }

    /// The model an alias stands for, or `name` itself.
    pub fn resolve<'a>(&'a self, name: &'a str) -> &'a str {
        self.aliases
            .iter()
            .find(|(alias, _)| alias.eq_ignore_ascii_case(name.trim()))
            .map_or(name.trim(), |(_, id)| id.as_str())
    }

    /// Which of two cached files is preferred: the CLI's surface, then the latest fetched.
    fn rank(&self) -> (bool, Option<DateTime<Utc>>) {
        match &self.source {
            Source::Cached { surface, fetched } => (surface == CLI_SURFACE, *fetched),
            Source::BuiltIn(_) => (false, None),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::files::tests::Scratch;

    /// A synthetic file in the shape Claude Code caches (format 2), its ids made up but for the
    /// two the aliases name.
    fn catalog(surface: &str, fetched: i64, first: &str) -> String {
        serde_json::json!({
            "version": 2,
            "fetchedAt": fetched,
            "staleAt": fetched + 3_600_000,
            "catalog": {
                "surface": surface,
                "config": {
                    "models": [
                        {"id": "model-old", "name": "Old", "section": "overflow",
                         "runtime": {"effort_levels": ["low", "medium", "high", "max"],
                                     "default_effort": "high"}},
                        {"id": first, "name": "First", "section": "main",
                         "runtime": {"effort_levels": ["low", "medium", "high", "xhigh", "max"],
                                     "default_effort": "medium"},
                         "min_claude_code_version": "2.1.280"},
                        {"id": "model-plain", "section": "main"},
                        {"name": "no id"}
                    ],
                    "provider_alias_targets": {
                        "first": {"default": first, "per_provider": {"elsewhere": "other"}},
                        "broken": {"per_provider": {}}
                    }
                },
                "state": {"model": first}
            }
        })
        .to_string()
    }

    #[test]
    fn reads_the_models_in_their_sections() {
        let parsed = Catalog::parse(&catalog("cc", 1_793_638_097_074, "model-first")).unwrap();
        let ids: Vec<&str> = parsed.models.iter().map(|m| m.id.as_str()).collect();
        assert_eq!(ids, ["model-first", "model-plain", "model-old"]);
        let first = &parsed.models[0];
        assert!(first.main);
        assert_eq!(first.efforts.len(), 5);
        assert_eq!(first.default_effort.as_deref(), Some("medium"));
        assert_eq!(first.min_cli, Some(Version::new(2, 1, 280)));
        let plain = parsed.model("model-plain").unwrap();
        assert_eq!(plain.name, "model-plain");
        assert!(plain.efforts.is_empty());
        assert_eq!(
            parsed.aliases,
            [("first".to_owned(), "model-first".to_owned())]
        );
        assert_eq!(
            parsed.model("FIRST").map(|m| m.name.as_str()),
            Some("First")
        );
        assert_eq!(parsed.resolve("model-old"), "model-old");
        assert!(matches!(
            parsed.source,
            Source::Cached { ref surface, fetched: Some(_) } if surface == "cc"
        ));
    }

    #[test]
    fn prefers_the_clis_surface_then_the_latest() {
        let scratch = Scratch::new("catalog-pick");
        let write = |name: &str, text: &str| fs::write(scratch.0.join(name), text).unwrap();
        write("a-ccd.json", &catalog("ccd", 3_000, "model-desktop-new"));
        write("b-cc.json", &catalog("cc", 1_000, "model-cli-old"));
        write("c-cc.json", &catalog("cc", 2_000, "model-cli-new"));
        write("d-cc.json", "{ not json");
        write(
            "e-cc.json",
            &catalog("cc", 9_000, "x").replace("\"version\":2", "\"version\":3"),
        );
        write("notes.txt", "not a catalog");
        let read = Catalog::read(&scratch.0);
        assert_eq!(read.models[0].id, "model-cli-new");
        // Only desktop files: the latest of them.
        let desktop = Scratch::new("catalog-desktop");
        fs::write(
            desktop.0.join("x-ccd.json"),
            catalog("ccd", 5, "model-desktop"),
        )
        .unwrap();
        assert_eq!(Catalog::read(&desktop.0).models[0].id, "model-desktop");
    }

    #[test]
    fn falls_back_to_the_built_in_list() {
        let scratch = Scratch::new("catalog-none");
        let empty = Catalog::read(&scratch.0);
        assert_eq!(
            empty.source,
            Source::BuiltIn("no cached catalog".to_owned())
        );
        assert_eq!(
            empty.model("fable").map(|m| m.id.as_str()),
            Some("claude-fable-5-1")
        );
        assert_eq!(empty.models.len(), 4);
        fs::write(scratch.0.join("x-cc.json"), "[]").unwrap();
        assert!(matches!(
            Catalog::read(&scratch.0).source,
            Source::BuiltIn(why) if why.contains("format unknown")
        ));
        assert!(matches!(
            Catalog::read(&scratch.0.join("missing")).source,
            Source::BuiltIn(why) if why.starts_with("no catalog folder")
        ));
        let no_models = serde_json::json!({"version": 2, "catalog": {"config": {"models": []}}});
        assert!(Catalog::parse(&no_models.to_string()).is_err());
    }
}
