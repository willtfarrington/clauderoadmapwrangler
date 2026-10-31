//! Tool versions: the oldest Claude Code CLI the tool drives (D-15) and the reading of a version
//! out of a tool's `--version` output or a folder name.

use std::fmt;
use std::str::FromStr;

/// The oldest Claude Code CLI the tool drives (D-15): headless launches are refused below it. A
/// brief raises it only when it needs a newer flag; features are detected, never assumed.
pub const MIN_CLAUDE_CODE: Version = Version::new(2, 1, 261);

/// A `major.minor.patch` version; compares field by field.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Version {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
}

impl Version {
    pub const fn new(major: u32, minor: u32, patch: u32) -> Self {
        Self {
            major,
            minor,
            patch,
        }
    }

    /// The first `x.y.z` in `text`, wherever it sits: `2.1.261 (Claude Code)`, `gh version 2.103.3
    /// (2026-10-26)`, `anthropic.claude-code-2.1.314-win32-x64`.
    pub fn find_in(text: &str) -> Option<Self> {
        text.split(|c: char| !(c.is_ascii_digit() || c == '.'))
            .find_map(|token| token.trim_matches('.').parse().ok())
    }
}

impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

/// The text was not exactly three dot-separated numbers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseVersionError(String);

impl fmt::Display for ParseVersionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "not a major.minor.patch version: {:?}", self.0)
    }
}

impl std::error::Error for ParseVersionError {}

impl FromStr for Version {
    type Err = ParseVersionError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let numbers: Vec<u32> = s
            .split('.')
            .map(str::parse)
            .collect::<Result<_, _>>()
            .map_err(|_| ParseVersionError(s.to_owned()))?;
        match numbers[..] {
            [major, minor, patch] => Ok(Self::new(major, minor, patch)),
            _ => Err(ParseVersionError(s.to_owned())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_the_version_in_each_tools_output() {
        let cases = [
            ("2.1.261 (Claude Code)\n", Version::new(2, 1, 261)),
            (
                "gh version 2.103.3 (2026-10-26)\nhttps://github.com/cli/cli/releases/tag/v2.103.3\n",
                Version::new(2, 103, 3),
            ),
            (
                "1.104.2\n  tailscale commit: 0f3c9e1a\n  go version: go1.25.4\n",
                Version::new(1, 104, 2),
            ),
            (
                "1.144.0\n8d1a7f2c9b3e4f5a6d7c8b9a0e1f2d3c4b5a6978\nx64\n",
                Version::new(1, 144, 0),
            ),
            (
                "anthropic.claude-code-2.1.314-win32-x64",
                Version::new(2, 1, 314),
            ),
            ("v10.0.1-rc.2", Version::new(10, 0, 1)),
        ];
        for (text, expected) in cases {
            assert_eq!(Version::find_in(text), Some(expected), "{text:?}");
        }
    }

    #[test]
    fn finds_nothing_without_three_numbers() {
        for text in ["", "Claude Code", "1.2", "version 3", "...", "1.2.3.4"] {
            assert_eq!(Version::find_in(text), None, "{text:?}");
        }
    }

    #[test]
    fn parses_exactly_three_numbers() {
        assert_eq!("2.1.261".parse(), Ok(Version::new(2, 1, 261)));
        for text in [
            "2.1",
            "2.1.261.0",
            "2.1.x",
            "2..1",
            " 2.1.261",
            "99999999999.0.0",
        ] {
            assert!(text.parse::<Version>().is_err(), "{text:?}");
        }
    }

    #[test]
    fn orders_numerically_not_textually() {
        assert!(Version::new(2, 1, 261) < Version::new(2, 1, 314));
        assert!(Version::new(2, 1, 314) < Version::new(2, 2, 0));
        assert!(Version::new(2, 9, 0) < Version::new(2, 10, 0));
        assert!(Version::new(9, 99, 99) < Version::new(10, 0, 0));
    }

    #[test]
    fn minimum_is_the_planned_cli() {
        assert_eq!(MIN_CLAUDE_CODE.to_string(), "2.1.261");
        assert!(Version::new(2, 1, 314) >= MIN_CLAUDE_CODE);
        assert!(Version::new(2, 1, 260) < MIN_CLAUDE_CODE);
    }
}
