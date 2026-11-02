//! The badges of a project's row (D-24, D-25): what it is built with ([`stack`]), how lively its
//! repository is ([`activity`]), where it is hosted and whether that copy is current
//! ([`hosting`]), whether its CI passes and it carries a license ([`health`]), and how old its
//! last backup is ([`backup`]). The files and the backup job are read with no child, git one
//! child at a time, GitHub through `gh` daily and on demand ([`online`]), its answers kept in the
//! ledger's cache with the time they were fetched.

pub mod activity;
pub mod backup;
pub mod health;
pub mod hosting;
pub mod online;
mod psdata;
pub mod stack;

use chrono::TimeDelta;

/// An age in the row's few characters: `now`, `40 min`, `5 h`, `3 d`, `6 w`, `4 mo`, `2 y`.
pub fn short_age(age: TimeDelta) -> String {
    let minutes = age.num_minutes().max(0);
    match minutes {
        0..2 => "now".to_owned(),
        2..60 => format!("{minutes} min"),
        60..2880 => format!("{} h", minutes / 60),
        _ => match age.num_days() {
            days @ 2..14 => format!("{days} d"),
            days @ 14..60 => format!("{} w", days / 7),
            days @ 60..730 => format!("{} mo", days / 30),
            days => format!("{} y", days / 365),
        },
    }
}

/// Times as RFC 3339 text in UTC, as the ledger keeps them (its cache holds these readings as
/// JSON).
pub(crate) mod rfc3339 {
    use chrono::{DateTime, SecondsFormat, Utc};
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S: Serializer>(at: &Option<DateTime<Utc>>, s: S) -> Result<S::Ok, S::Error> {
        match at {
            Some(at) => s.serialize_some(&at.to_rfc3339_opts(SecondsFormat::Secs, true)),
            None => s.serialize_none(),
        }
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<Option<DateTime<Utc>>, D::Error> {
        Option::<String>::deserialize(d)?
            .map(|text| {
                parse(&text).ok_or_else(|| serde::de::Error::custom("not an RFC 3339 time"))
            })
            .transpose()
    }

    /// An RFC 3339 time in any offset, as UTC.
    pub fn parse(text: &str) -> Option<DateTime<Utc>> {
        DateTime::parse_from_rfc3339(text.trim())
            .ok()
            .map(|at| at.with_timezone(&Utc))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ages_read_short() {
        let cases = [
            (TimeDelta::seconds(-30), "now"),
            (TimeDelta::seconds(90), "now"),
            (TimeDelta::minutes(40), "40 min"),
            (TimeDelta::hours(5), "5 h"),
            (TimeDelta::hours(47), "47 h"),
            (TimeDelta::days(3), "3 d"),
            (TimeDelta::days(20), "2 w"),
            (TimeDelta::days(130), "4 mo"),
            (TimeDelta::days(800), "2 y"),
        ];
        for (age, text) in cases {
            assert_eq!(short_age(age), text, "{age:?}");
        }
    }

    #[test]
    fn times_parse_in_any_offset() {
        let at = rfc3339::parse("2026-11-01T12:31:20.9147466-05:00").unwrap();
        assert_eq!(at.to_rfc3339(), "2026-11-01T17:31:20.914746600+00:00");
        assert_eq!(rfc3339::parse("yesterday"), None);
    }
}
