//! The usage's attribution (D-23; EP-18a). Each reading of the endpoint's, set against the
//! window's previous one, gives each figure's rise, which is split equally among the sessions that
//! worked between the two. The window's first reading after its start splits nothing, since
//! nothing watched before it. A part goes to its session's latest ledger row
//! (`Ledger::add_share`); a session with no row (outside the registry, or one that never earned
//! it) keeps its part unattributed, so that the registered projects' sums never take it in.

use chrono::{DateTime, TimeDelta, Utc};

use crate::ledger::{Part, Sample};

/// A weekly window's length.
pub const WEEK: TimeDelta = TimeDelta::days(7);

/// One figure at a reading: its share in percent and its window's reset.
type At = (Option<f64>, Option<DateTime<Utc>>);

/// One figure's rise from `before` to `after`: the difference within one limit window, or the new
/// figure once the window has reset between them; none when either reading lacks the figure.
pub fn rise(before: At, after: At) -> Option<f64> {
    let (from, to) = (before.0?, after.0?);
    if super::same_reset(before.1, after.1) {
        Some((to - from).max(0.0))
    } else {
        Some(to.max(0.0))
    }
}

/// Each worker's part of the rise from `before` to `after`, two endpoint readings: an equal share
/// of each figure's rise, with the figures at the two readings; none when no session worked.
pub fn part(before: &Sample, after: &Sample, workers: usize) -> Option<Part> {
    if workers == 0 {
        return None;
    }
    let each = |rise: Option<f64>| rise.map(|rise| rise / workers as f64);
    let weekly = rise(
        (before.weekly_percent, before.weekly_resets_at),
        (after.weekly_percent, after.weekly_resets_at),
    );
    let fable = rise(
        (before.fable_percent, before.fable_resets_at),
        (after.fable_percent, after.fable_resets_at),
    );
    Some(Part {
        weekly: each(weekly),
        fable: each(fable),
        weekly_before: before.weekly_percent,
        weekly_after: after.weekly_percent,
        fable_before: before.fable_percent,
        fable_after: after.fable_percent,
    })
}

/// The start of the weekly window that `resets_at` closes, as of `now`: seven days before the
/// next reset, a reset already past moving on a week at a time.
pub fn week_start(resets_at: DateTime<Utc>, now: DateTime<Utc>) -> DateTime<Utc> {
    let mut reset = resets_at;
    if reset <= now {
        let weeks = (now - reset).num_seconds() / WEEK.num_seconds() + 1;
        reset += TimeDelta::seconds(WEEK.num_seconds().saturating_mul(weeks));
    }
    reset - WEEK
}

#[cfg(test)]
mod tests {
    use super::*;

    fn at(text: &str) -> DateTime<Utc> {
        DateTime::parse_from_rfc3339(text)
            .unwrap()
            .with_timezone(&Utc)
    }

    fn sample(weekly: Option<f64>, fable: Option<f64>, resets: &str) -> Sample {
        Sample {
            taken_at: at("2026-11-05T00:00:00Z"),
            source: "endpoint".to_owned(),
            session_id: None,
            weekly_percent: weekly,
            weekly_resets_at: Some(at(resets)),
            fable_percent: fable,
            fable_resets_at: Some(at(resets)),
            five_hour_percent: None,
            five_hour_resets_at: None,
        }
    }

    const RESET: &str = "2026-11-07T17:00:00Z";
    const NEXT: &str = "2026-11-14T17:00:00Z";

    #[test]
    fn a_rise_is_the_change_within_a_window_or_the_new_figure_after_a_reset() {
        let reset = Some(at(RESET));
        assert_eq!(rise((Some(27.0), reset), (Some(28.0), reset)), Some(1.0));
        assert_eq!(rise((Some(28.0), reset), (Some(28.0), reset)), Some(0.0));
        assert_eq!(rise((Some(28.0), reset), (Some(27.0), reset)), Some(0.0));
        assert_eq!(
            rise((Some(96.0), reset), (Some(2.0), Some(at(NEXT)))),
            Some(2.0)
        );
        assert_eq!(rise((None, reset), (Some(2.0), reset)), None);
        assert_eq!(rise((Some(2.0), reset), (None, reset)), None);
    }

    #[test]
    fn the_rise_is_shared_equally_among_the_sessions_that_worked() {
        let before = sample(Some(27.0), Some(19.0), RESET);
        let after = sample(Some(28.0), Some(19.0), RESET);
        assert_eq!(part(&before, &after, 0), None);
        let third = part(&before, &after, 3).unwrap();
        assert!((third.weekly.unwrap() - 1.0 / 3.0).abs() < 1e-9);
        assert_eq!(third.fable, Some(0.0));
        assert_eq!(
            (third.weekly_before, third.weekly_after),
            (Some(27.0), Some(28.0))
        );
        assert_eq!(
            (third.fable_before, third.fable_after),
            (Some(19.0), Some(19.0))
        );
        let no_fable = part(&before, &sample(Some(29.0), None, RESET), 1).unwrap();
        assert_eq!((no_fable.weekly, no_fable.fable), (Some(2.0), None));
    }

    #[test]
    fn the_week_starts_seven_days_before_its_next_reset() {
        let now = at("2026-11-05T00:30:00Z");
        assert_eq!(week_start(at(RESET), now), at("2026-10-31T17:00:00Z"));
        // A cached reset already past: the window it opened.
        assert_eq!(
            week_start(at("2026-11-04T17:00:00Z"), now),
            at("2026-11-04T17:00:00Z")
        );
        assert_eq!(
            week_start(at("2026-10-20T17:00:00Z"), now),
            at("2026-11-03T17:00:00Z")
        );
    }
}
