//! The roadmap-wrangler plugin (D-27; EP-22): its files as Claude Code reads them, and the
//! engine's texts its reference quotes. A structural check of the repository's own files, since
//! tests never run the real CLI (D-29); `claude plugin validate` is a session's check by hand.

use std::fs;
use std::path::{Path, PathBuf};

use crw_core::formalise;
use crw_core::inbox::mailman;
use crw_core::plugin;

/// The skills the plugin ships (D-27).
const SKILLS: [&str; 4] = ["formalise", "plan", "replan", "status"];

/// Claude Code's cap on a skill's `description` and `when_to_use` together in its skill listing,
/// in characters (its skills page, read 2026-11-10).
const LISTING_CAP: usize = 1_536;

/// How a skill names a file of its plugin; Claude Code puts the plugin's folder in its place.
const ROOT: &str = "${CLAUDE_PLUGIN_ROOT}/";

fn repository() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join("..")
}

fn plugin() -> PathBuf {
    repository().join("plugins").join("roadmap-wrangler")
}

fn skill(name: &str) -> String {
    read(&plugin().join("skills").join(name).join("SKILL.md"))
}

/// A file's text with its line endings made `\n`, whatever the checkout made them.
fn read(path: &Path) -> String {
    fs::read_to_string(path)
        .unwrap_or_else(|e| panic!("{}: {e}", path.display()))
        .replace("\r\n", "\n")
}

fn json(path: &Path) -> serde_json::Value {
    serde_json::from_str(&read(path)).unwrap_or_else(|e| panic!("{}: {e}", path.display()))
}

/// A skill's front matter, each line's key and value, a quoted value without its quotes.
fn front_matter(text: &str) -> Vec<(String, String)> {
    let body = text
        .strip_prefix("---\n")
        .expect("a skill opens with front matter");
    let end = body.find("\n---\n").expect("its front matter closes");
    body[..end]
        .lines()
        .filter_map(|line| {
            let (key, value) = line.split_once(':')?;
            let value = value.trim();
            let value = value
                .strip_prefix('"')
                .and_then(|value| value.strip_suffix('"'))
                .unwrap_or(value);
            Some((key.trim().to_owned(), value.to_owned()))
        })
        .collect()
}

#[test]
fn the_manifest_names_the_plugin_and_the_marketplace_lists_it() {
    let manifest = json(&plugin().join(".claude-plugin").join("plugin.json"));
    assert_eq!(manifest["name"], "roadmap-wrangler");
    for key in [
        "displayName",
        "description",
        "license",
        "homepage",
        "repository",
    ] {
        assert!(manifest[key].is_string(), "plugin.json has no {key}");
    }
    assert!(manifest["author"]["name"].is_string());

    let marketplace = json(&repository().join(".claude-plugin").join("marketplace.json"));
    assert_eq!(marketplace["name"], "clauderoadmapwrangler");
    assert!(marketplace["owner"]["name"].is_string());
    let entries = marketplace["plugins"].as_array().expect("a plugins array");
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0]["name"], "roadmap-wrangler");
    assert_eq!(entries[0]["source"], "./plugins/roadmap-wrangler");
}

#[test]
fn each_skill_is_named_for_its_folder_and_says_when_to_use_it() {
    let mut folders: Vec<String> = fs::read_dir(plugin().join("skills"))
        .unwrap()
        .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    folders.sort();
    assert_eq!(folders, SKILLS);
    for name in SKILLS {
        let fields = front_matter(&skill(name));
        let field = |key: &str| {
            fields
                .iter()
                .find(|(k, _)| k == key)
                .map_or("", |(_, value)| value.as_str())
        };
        assert_eq!(field("name"), name);
        for key in ["description", "when_to_use"] {
            // A block scalar's text is on the lines after its key, which this reading would miss.
            assert!(
                !matches!(field(key), "" | "|" | ">" | "|-" | ">-"),
                "{name} has no {key} on its line"
            );
        }
        let listing = format!("{}{}", field("description"), field("when_to_use"));
        assert!(
            listing.chars().count() <= LISTING_CAP,
            "{name}'s listing is cut at {LISTING_CAP} characters"
        );
        // Claude Code shows angle brackets in a listing as entities.
        assert!(!listing.contains(['<', '>']), "{name}'s listing has <>");
    }
}

#[test]
fn every_file_a_skill_names_in_the_plugin_is_there() {
    for name in SKILLS {
        let text = skill(name);
        let mut named = 0;
        for (at, _) in text.match_indices(ROOT) {
            let rest = &text[at + ROOT.len()..];
            let end = rest
                .find(|c: char| c.is_whitespace() || matches!(c, '`' | ')' | ',' | '"'))
                .unwrap_or(rest.len());
            let path = rest[..end].trim_end_matches(['.', ':']);
            // Claude Code refuses a path out of the plugin's folder, which an installed copy is.
            assert!(
                !path.split(['/', '\\']).any(|part| part == ".."),
                "{name} names {path}, out of the plugin"
            );
            assert!(plugin().join(path).is_file(), "{name} names {path}");
            named += 1;
        }
        assert!(named > 0, "{name} does not point at the reference");
    }
}

/// Every file under `plugins/roadmap-wrangler/`, by its path under the folder with `/`.
fn files_of(folder: &Path) -> Vec<(String, PathBuf)> {
    let mut found = Vec::new();
    let mut pending = vec![folder.to_path_buf()];
    while let Some(dir) = pending.pop() {
        for entry in fs::read_dir(&dir).unwrap().flatten() {
            let path = entry.path();
            if path.is_dir() {
                pending.push(path);
            } else {
                let name = path
                    .strip_prefix(folder)
                    .unwrap()
                    .components()
                    .map(|c| c.as_os_str().to_string_lossy().into_owned())
                    .collect::<Vec<_>>()
                    .join("/");
                found.push((name, path));
            }
        }
    }
    found.sort();
    found
}

/// The build carries the plugin's folder whole (EP-22a): the table names every file there and
/// nothing else, each byte for byte, so a run's copy is the folder as committed.
#[test]
fn the_build_carries_every_file_of_the_plugin() {
    let files = files_of(&plugin());
    let carried: Vec<&str> = plugin::FILES.iter().map(|(name, _)| *name).collect();
    let on_disk: Vec<&str> = files.iter().map(|(name, _)| name.as_str()).collect();
    assert_eq!(
        carried, on_disk,
        "crw_core::plugin::FILES must list the folder's files, in sorted order"
    );
    for ((name, text), (_, path)) in plugin::FILES.iter().zip(&files) {
        assert_eq!(
            fs::read(path).unwrap(),
            text.as_bytes(),
            "{name} is carried as another text"
        );
    }
    assert!(carried.contains(&".claude-plugin/plugin.json"));
    for skill in SKILLS {
        assert!(carried.contains(&format!("skills/{skill}/SKILL.md").as_str()));
    }
}

#[test]
fn the_reference_quotes_the_engines_texts_byte_for_byte() {
    let reference = read(&plugin().join("reference").join("house-style.md"));
    for (what, text) in [
        ("the rule block", formalise::block("\n")),
        (
            "the owner tasks' header",
            formalise::OWNER_TASKS_HEADER.to_owned(),
        ),
        ("the inbox's header", mailman::HEADER.to_owned()),
        ("the brief template", formalise::TEMPLATE.to_owned()),
    ] {
        assert!(
            reference.contains(&text),
            "the reference no longer quotes {what} as crw formalise writes it"
        );
    }
}
