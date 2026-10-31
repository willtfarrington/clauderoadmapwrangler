//! `crw`, the console side of clauderoadmapwrangler (D-2, D-14): `doctor` from EP-1; `scan`,
//! `inbox send` and the `hook` and `statusline` relays arrive with their briefs.

use std::process::ExitCode;

use crw_core::doctor::{self, Outcome};
use crw_core::version::MIN_CLAUDE_CODE;

const USAGE: &str = "\
usage: crw <command>

commands:
  doctor      print the versions of claude, gh, tailscale, code and the VS Code extension
  help        print this text
  --version   print crw's version";

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let args: Vec<&str> = args.iter().map(String::as_str).collect();
    match args.as_slice() {
        ["doctor"] => doctor(),
        ["--version" | "-V"] => {
            println!("crw {}", env!("CARGO_PKG_VERSION"));
            ExitCode::SUCCESS
        }
        [] | ["help" | "--help" | "-h"] => {
            println!("{USAGE}");
            ExitCode::SUCCESS
        }
        _ => {
            eprintln!("crw: unknown command: {}\n\n{USAGE}", args.join(" "));
            ExitCode::from(2)
        }
    }
}

/// Prints one line per tool and fails when Claude Code is missing or older than the minimum.
fn doctor() -> ExitCode {
    println!("crw {} doctor", env!("CARGO_PKG_VERSION"));
    let checks = doctor::run();
    for check in &checks {
        let (version, detail) = match &check.outcome {
            Outcome::Found { version, source } => {
                (version.to_string(), source.display().to_string())
            }
            Outcome::Missing => ("-".to_owned(), "not found".to_owned()),
            Outcome::Failed { source, reason } => {
                ("?".to_owned(), format!("{reason}: {}", source.display()))
            }
        };
        println!("  {:<10} {:<9} {detail}", check.name, version);
    }
    if doctor::claude_meets_minimum(&checks) {
        println!("claude meets the minimum version {MIN_CLAUDE_CODE}");
        ExitCode::SUCCESS
    } else {
        println!(
            "claude is missing or older than {MIN_CLAUDE_CODE}: headless sessions will be refused"
        );
        ExitCode::FAILURE
    }
}
