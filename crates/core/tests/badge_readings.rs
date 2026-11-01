//! The badges' readings on scratch repositories (EP-7): the head check against an origin that is
//! a bare repository in the same scratch folder, so nothing reaches the network; git only, never
//! `gh` and never the `claude` CLI (D-29).

use std::fs;
use std::path::{Path, PathBuf};
use std::thread;

use crw_core::badges::activity::Activity;
use crw_core::badges::hosting::{self, Hosting, Verdict};
use crw_core::{git, process};

/// A scratch folder holding the repositories, removed when dropped.
struct Scratch(PathBuf);

impl Scratch {
    fn new(name: &str) -> Self {
        let dir = std::env::temp_dir().join(format!("crw-test-{}-{name}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        Self(dir)
    }

    /// Runs git in `dir` with a neutral identity and no signing or line-ending conversion, spaced
    /// like every child the tool starts.
    fn git(&self, dir: &str, args: &[&str]) -> String {
        thread::sleep(process::SPACING);
        let folder = self.0.join(dir);
        let mut all = vec![
            "-C",
            folder.to_str().unwrap(),
            "-c",
            "user.name=crw-test",
            "-c",
            "user.email=crw-test@example.invalid",
            "-c",
            "commit.gpgsign=false",
            "-c",
            "core.autocrlf=false",
        ];
        all.extend_from_slice(args);
        process::output_of(Path::new("git"), all, process::TIMEOUT)
            .unwrap_or_else(|e| panic!("git {args:?} in {dir}: {e}"))
    }

    fn commit(&self, dir: &str, file: &str) {
        fs::write(self.0.join(dir).join(file), file).unwrap();
        self.git(dir, &["add", "-A"]);
        self.git(dir, &["commit", "-q", "-m", file]);
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[test]
fn the_head_check_follows_the_origin() {
    let scratch = Scratch::new("badges-head");
    let origin = scratch.0.join("origin.git");
    let origin_url = origin.to_str().unwrap();
    scratch.git(".", &["init", "-q", "--bare", "-b", "main", "origin.git"]);
    scratch.git(".", &["init", "-q", "-b", "main", "work"]);
    scratch.commit("work", "one.txt");
    scratch.git("work", &["remote", "add", "origin", origin_url]);
    scratch.git("work", &["push", "-q", "-u", "origin", "main"]);
    let work = scratch.0.join("work");

    // Pushed: the two heads are the same commit.
    let check = hosting::check_head(&work);
    assert_eq!(check.remote_ref, "refs/heads/main");
    assert_eq!(check.verdict, Verdict::InSync, "{check:?}");
    assert_eq!(check.remote, check.local);

    // A local commit not pushed: ahead, by the head check and by git's own counts.
    scratch.commit("work", "two.txt");
    assert_eq!(hosting::check_head(&work).verdict, Verdict::Ahead(1));
    thread::sleep(process::SPACING);
    let status = git::status(&work).unwrap();
    assert_eq!(status.ahead_behind, Some((1, 0)));
    assert_eq!(status.upstream_parts(), Some(("origin", "main")));
    assert_eq!(status.changes, []);
    let activity = Activity::read(&status, None);
    assert_eq!(activity.branch_text(), "main → origin/main, 1 ahead");

    // Another clone pushes: the origin holds a commit this clone has never fetched.
    scratch.git(".", &["clone", "-q", origin_url, "other"]);
    scratch.commit("other", "three.txt");
    scratch.git("other", &["push", "-q", "origin", "main"]);
    assert_eq!(hosting::check_head(&work).verdict, Verdict::NotFetched);
    // git's own counts still say ahead: they compare with the origin as last fetched.
    thread::sleep(process::SPACING);
    assert_eq!(git::status(&work).unwrap().ahead_behind, Some((1, 0)));

    // Once fetched (by the owner, never by the tool), both sides are counted.
    scratch.git("work", &["fetch", "-q"]);
    assert_eq!(
        hosting::check_head(&work).verdict,
        Verdict::Diverged {
            ahead: 1,
            behind: 1
        }
    );
    thread::sleep(process::SPACING);
    let status = git::status(&work).unwrap();
    assert_eq!(status.ahead_behind, Some((1, 1)));

    // The origin is a folder, not GitHub, and is read from the config with no child.
    let url = git::remote_url(&work, "origin").unwrap();
    assert_eq!(Path::new(&url), origin);
    assert!(matches!(
        hosting::classify(&["work"], Some(&url), None),
        Hosting::Elsewhere(_)
    ));

    // A dirty tree counts its changed and untracked paths.
    fs::write(work.join("one.txt"), "changed").unwrap();
    fs::write(work.join("new.txt"), "new").unwrap();
    thread::sleep(process::SPACING);
    assert_eq!(git::status(&work).unwrap().changes.len(), 2);
}

#[test]
fn an_origin_that_cannot_be_reached_fails_the_check() {
    let scratch = Scratch::new("badges-unreachable");
    scratch.git(".", &["init", "-q", "-b", "main", "work"]);
    scratch.commit("work", "one.txt");
    let gone = scratch.0.join("gone.git");
    scratch.git("work", &["remote", "add", "origin", gone.to_str().unwrap()]);
    let check = hosting::check_head(&scratch.0.join("work"));
    assert_eq!(
        check.remote_ref, "HEAD",
        "no upstream: the origin's HEAD is asked"
    );
    assert!(
        matches!(&check.verdict, Verdict::Failed(reason) if reason.starts_with("git ls-remote")),
        "{check:?}"
    );
    assert!(check.local.is_some());
}
