//! Notifications (D-21; EP-15): which moments reach the owner, and how. A moment that needs the
//! owner or ends a session's work is a toast at once, in the window's own popup: a session waits
//! for an answer, has finished, has failed, and the account's weekly or Fable usage reaches 80 %
//! or 95 % (EP-17, once per mark and limit window), and a chain's state changes (EP-19a: a brief
//! chained, the chain held or ended). A moment that only wants a look is a badge on the tray icon: a new unadopted folder, an orphan or a mismatched
//! remote, a stale backup, an inbox arrival. Quiet hours, kept in the registry, hold the toasts
//! back; the badge still counts. Each toast, toast held back, badge arrival and Show is a line of
//! the ledger's `event` table of kind [`EVENT_KIND`], worded by [`line`]. The window shows them
//! (`app::toasts`, `app::popup`, `app::tray`); this module keeps the rules, and EP-20 adds ntfy.

use std::fmt;

use chrono::{NaiveTime, Timelike};
use serde::Deserialize;

/// The ledger's `event` kind of the notifications' log.
pub const EVENT_KIND: &str = "notification";

/// What happened, as the rules see it (D-21).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Kind {
    /// A session waits for the owner: a question, a permission, a run deferred with a request open.
    Waiting,
    /// A session's work is done: a run's result, a long turn's end, a background session done.
    Finished,
    /// A run, or a background session, failed.
    Failed,
    /// The weekly or the Fable-scoped usage reached 80 % or 95 % (D-23; EP-17).
    Usage,
    /// A folder of the data root that is neither registered nor ignored.
    Unadopted,
    /// A remote that needs a look: an orphan, a missing repository, an origin diverged or ahead
    /// with commits never fetched.
    Remote,
    /// A backup that is stale, failing, or of a folder no set covers.
    Backup,
    /// An open message in a project's inbox.
    Inbox,
    /// A chain's state changed (D-22; EP-19a): a brief chained, the chain held, or ended.
    Chain,
}

/// How a kind reaches the owner.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Channel {
    /// A Windows toast at once, outside the quiet hours.
    Toast,
    /// The tray icon's mark and a line in its menu, nothing more.
    Badge,
}

impl Kind {
    pub const ALL: [Kind; 9] = [
        Kind::Waiting,
        Kind::Finished,
        Kind::Failed,
        Kind::Usage,
        Kind::Chain,
        Kind::Unadopted,
        Kind::Remote,
        Kind::Backup,
        Kind::Inbox,
    ];

    /// The rule table (D-21).
    pub fn channel(self) -> Channel {
        match self {
            Kind::Waiting | Kind::Finished | Kind::Failed | Kind::Usage | Kind::Chain => {
                Channel::Toast
            }
            Kind::Unadopted | Kind::Remote | Kind::Backup | Kind::Inbox => Channel::Badge,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Kind::Waiting => "waiting",
            Kind::Finished => "finished",
            Kind::Failed => "failed",
            Kind::Usage => "usage",
            Kind::Chain => "chain",
            Kind::Unadopted => "unadopted",
            Kind::Remote => "remote",
            Kind::Backup => "backup",
            Kind::Inbox => "inbox",
        }
    }
}

/// The hours toasts are held back (D-21), in local time, from the first time up to the second:
/// `22:00-07:00` runs past midnight. The registry's `settings.quiet_hours`; none by default.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(try_from = "String")]
pub struct QuietHours {
    from: NaiveTime,
    to: NaiveTime,
}

impl QuietHours {
    /// Reads `HH:MM-HH:MM`; the two times must differ.
    pub fn parse(text: &str) -> Result<Self, String> {
        let wrong = || {
            format!(
                "quiet_hours takes HH:MM-HH:MM in local time, such as 22:00-07:00, not {text:?}"
            )
        };
        let (from, to) = text.split_once('-').ok_or_else(wrong)?;
        let time =
            |part: &str| NaiveTime::parse_from_str(part.trim(), "%H:%M").map_err(|_| wrong());
        let (from, to) = (time(from)?, time(to)?);
        if from == to {
            return Err(format!(
                "quiet_hours needs two different times, not {text:?}"
            ));
        }
        Ok(Self { from, to })
    }

    /// Whether `at` falls in them.
    pub fn contains(&self, at: NaiveTime) -> bool {
        if self.from < self.to {
            self.from <= at && at < self.to
        } else {
            at >= self.from || at < self.to
        }
    }
}

impl fmt::Display for QuietHours {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{:02}:{:02}-{:02}:{:02}",
            self.from.hour(),
            self.from.minute(),
            self.to.hour(),
            self.to.minute()
        )
    }
}

impl TryFrom<String> for QuietHours {
    type Error = String;

    fn try_from(text: String) -> Result<Self, String> {
        Self::parse(&text)
    }
}

/// What became of a notification, as its line in the log says.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Fate {
    /// Shown in the window's popup.
    Shown,
    /// Held back by the quiet hours.
    Quiet,
    /// Held back: the window was in front with the focus, and its status bar said it.
    InFront,
    /// Held back: the same session's wait was shown a moment before.
    Repeat,
    /// The owner opened it: the popup's Show, which leads where it says.
    Clicked,
    /// A badge event that arrived: the tray icon's mark.
    Badge,
}

impl Fate {
    pub fn words(&self) -> &'static str {
        match self {
            Fate::Shown => "popup",
            Fate::Quiet => "quiet hours",
            Fate::InFront => "in front",
            Fate::Repeat => "repeat",
            Fate::Clicked => "opened",
            Fate::Badge => "badge",
        }
    }
}

/// A notification's line in the log, its parts apart by ` · `: `popup · waiting · harbor EP-4
/// waits for you · Which colour?`. `what` is a [`Kind`]'s word, or `test` or `tray`.
pub fn line(fate: &Fate, what: &str, title: &str, text: &str) -> String {
    [fate.words(), what, title, text]
        .into_iter()
        .map(str::trim)
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join(" · ")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn at(text: &str) -> NaiveTime {
        NaiveTime::parse_from_str(text, "%H:%M").unwrap()
    }

    #[test]
    fn the_rule_table_toasts_what_needs_the_owner() {
        let toasts: Vec<Kind> = Kind::ALL
            .into_iter()
            .filter(|kind| kind.channel() == Channel::Toast)
            .collect();
        assert_eq!(
            toasts,
            [
                Kind::Waiting,
                Kind::Finished,
                Kind::Failed,
                Kind::Usage,
                Kind::Chain
            ]
        );
        assert_eq!(Kind::Chain.as_str(), "chain");
        for kind in [Kind::Unadopted, Kind::Remote, Kind::Backup, Kind::Inbox] {
            assert_eq!(kind.channel(), Channel::Badge, "{kind:?}");
        }
    }

    #[test]
    fn quiet_hours_may_run_past_midnight() {
        let night = QuietHours::parse("22:00-07:00").unwrap();
        for (time, quiet) in [
            ("22:00", true),
            ("23:30", true),
            ("00:00", true),
            ("06:59", true),
            ("07:00", false),
            ("12:00", false),
            ("21:59", false),
        ] {
            assert_eq!(night.contains(at(time)), quiet, "{time}");
        }
        let day = QuietHours::parse(" 09:00 - 17:30 ").unwrap();
        assert!(day.contains(at("09:00")) && day.contains(at("17:29")));
        assert!(!day.contains(at("17:30")) && !day.contains(at("08:59")));
        assert_eq!(day.to_string(), "09:00-17:30");
        assert_eq!(night.to_string(), "22:00-07:00");
        assert_eq!(
            QuietHours::parse("9:05-17:00").unwrap().to_string(),
            "09:05-17:00"
        );
    }

    #[test]
    fn quiet_hours_refuse_what_they_cannot_read() {
        for text in [
            "",
            "none",
            "22-07",
            "22:00",
            "25:00-07:00",
            "22:00-07:60",
            "22:00-22:00",
        ] {
            assert!(QuietHours::parse(text).is_err(), "{text:?}");
        }
        assert!(
            QuietHours::parse("22:00-22:00")
                .unwrap_err()
                .contains("two different times")
        );
    }

    #[test]
    fn quiet_hours_read_from_toml() {
        #[derive(Deserialize)]
        struct Settings {
            quiet_hours: QuietHours,
        }
        let read: Settings = toml::from_str("quiet_hours = \"22:30-06:15\"").unwrap();
        assert_eq!(read.quiet_hours.to_string(), "22:30-06:15");
        assert!(toml::from_str::<Settings>("quiet_hours = \"late\"").is_err());
    }

    #[test]
    fn a_line_joins_its_parts() {
        assert_eq!(
            line(
                &Fate::Shown,
                "waiting",
                "harbor EP-4 waits for you",
                "Which colour?"
            ),
            "popup · waiting · harbor EP-4 waits for you · Which colour?"
        );
        assert_eq!(
            line(&Fate::Clicked, "test", "Roadmap Wrangler", ""),
            "opened · test · Roadmap Wrangler"
        );
        assert_eq!(
            line(&Fate::Badge, "inbox", "harbor IN-3", " "),
            "badge · inbox · harbor IN-3"
        );
    }
}
