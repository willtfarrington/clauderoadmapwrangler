//! The window's pace (EP-43): how long each pass takes and its parts, how long its frame took to
//! draw, and how long a phone's form waited for the window's hand. A form that waited [`SLOW`] or
//! more gets a line in the rolling log with what the window did meanwhile: the pass before, the
//! time between that pass and the one that took the form, and that pass's own time before it. A
//! pass or a frame that took [`SLOW`] or more gets a line of its own, at most one a minute, the
//! others counted in the next. The window's own reads of the ledger in its pass wait
//! [`LEDGER_WAIT`] at most for another connection's lock, and try again after [`LEDGER_RETRY`].

use std::time::{Duration, Instant};

/// A wait, a pass or a frame this long is worth a line (EP-43).
pub const SLOW: Duration = Duration::from_secs(2);

/// How long the window's reads of the ledger in its pass wait for another connection's lock,
/// each statement (EP-43): the lock of a commit, or of a writer waiting for a reader.
pub const LEDGER_WAIT: Duration = Duration::from_millis(250);

/// When a read that found the ledger locked tries again (EP-43).
pub const LEDGER_RETRY: Duration = Duration::from_secs(1);

/// A read found the ledger locked in this pass and waits for its retry: the pass's end asks for
/// a wake then, since a window with nothing else to do would sleep past it (the review's).
static RETRY_WANTED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// When a read that found the ledger locked at `now` tries again; the pass's end wakes the
/// window for it ([`retry_wanted`]).
pub fn retry_after(now: Instant) -> Instant {
    RETRY_WANTED.store(true, std::sync::atomic::Ordering::Relaxed);
    now + LEDGER_RETRY
}

/// Whether a read asked for a retry since the last call: taken once, at the pass's end.
pub fn retry_wanted() -> bool {
    RETRY_WANTED.swap(false, std::sync::atomic::Ordering::Relaxed)
}

/// At most one slow pass's or frame's line this often; the others are counted in the next.
const LINE_EVERY: Duration = Duration::from_secs(60);

/// A part this short is left out of a line.
const PART_FLOOR: Duration = Duration::from_millis(50);

/// How many parts a line names, the longest first.
const PARTS_NAMED: usize = 3;

/// A pass under way: when it began, when its last part ended, and its parts so far.
#[derive(Debug, Clone)]
struct Lap {
    began: Instant,
    at: Instant,
    parts: Vec<(&'static str, Duration)>,
}

/// A pass that ended: when it began and ended, and its parts.
#[derive(Debug, Clone)]
struct Done {
    began: Instant,
    ended: Instant,
    parts: Vec<(&'static str, Duration)>,
}

/// The window's passes as they go (EP-43).
#[derive(Debug, Default)]
pub struct Pace {
    current: Option<Lap>,
    last: Option<Done>,
    /// The last frame's drawing: when it ended and how long it took.
    drawn: Option<(Instant, Duration)>,
    /// When a slow line was last written, and the slow passes and frames since that had none.
    lined: Option<Instant>,
    unlined: usize,
}

impl Pace {
    /// A pass begins.
    pub fn begin(&mut self) {
        self.begin_at(Instant::now());
    }

    fn begin_at(&mut self, now: Instant) {
        self.current = Some(Lap {
            began: now,
            at: now,
            parts: Vec::new(),
        });
    }

    /// The pass's part `name` ended: the time since the part before is its own. Nothing outside a
    /// pass.
    pub fn mark(&mut self, name: &'static str) {
        self.mark_at(name, Instant::now());
    }

    fn mark_at(&mut self, name: &'static str, now: Instant) {
        if let Some(lap) = &mut self.current {
            lap.parts
                .push((name, now.saturating_duration_since(lap.at)));
            lap.at = now;
        }
    }

    /// The pass ends: its line when it was slow and a line is due.
    pub fn end(&mut self) -> Option<String> {
        self.end_at(Instant::now())
    }

    fn end_at(&mut self, now: Instant) -> Option<String> {
        let lap = self.current.take()?;
        let took = now.saturating_duration_since(lap.began);
        let words = format!(
            "The window's pass took {}{}.",
            secs(took),
            named(&lap.parts)
        );
        self.last = Some(Done {
            began: lap.began,
            ended: now,
            parts: lap.parts,
        });
        (took >= SLOW).then(|| self.due(now, words)).flatten()
    }

    /// The frame's drawing took `took`: its line when it was slow and a line is due.
    pub fn drawn(&mut self, took: Duration) -> Option<String> {
        self.drawn_at(took, Instant::now())
    }

    fn drawn_at(&mut self, took: Duration, now: Instant) -> Option<String> {
        self.drawn = Some((now, took));
        let words = format!("The window's frame took {} to draw.", secs(took));
        (took >= SLOW).then(|| self.due(now, words)).flatten()
    }

    /// What a form handed over at `sent` and taken or dropped now waited, in words that follow
    /// "A form from <caller>", when it waited [`SLOW`] or more.
    pub fn waited(&self, sent: Instant) -> Option<String> {
        self.waited_at(sent, Instant::now())
    }

    fn waited_at(&self, sent: Instant, now: Instant) -> Option<String> {
        let waited = now.saturating_duration_since(sent);
        if waited < SLOW {
            return None;
        }
        let mut clauses = Vec::new();
        match &self.last {
            Some(last) => {
                clauses.push(format!(
                    "the pass before took {}{}",
                    secs(last.ended.saturating_duration_since(last.began)),
                    named(&last.parts)
                ));
                if let Some(current) = &self.current {
                    let between = current.began.saturating_duration_since(last.ended);
                    let drawing = self
                        .drawn
                        .filter(|(ended, _)| *ended >= last.ended)
                        .map(|(_, took)| took);
                    clauses.push(match drawing {
                        Some(took) => format!(
                            "{} passed between it and this pass, {} of it the frame's drawing",
                            secs(between),
                            secs(took)
                        ),
                        None => format!(
                            "{} passed between it and this pass, with no frame drawn",
                            secs(between)
                        ),
                    });
                }
            }
            None => clauses.push("no pass before this one is on record".to_owned()),
        }
        if let Some(current) = &self.current {
            let before = now.saturating_duration_since(current.began);
            if before >= PART_FLOOR {
                clauses.push(format!(
                    "this pass took {} before the form{}",
                    secs(before),
                    named(&current.parts)
                ));
            }
        }
        Some(format!(
            "waited {} for the window: {}.",
            secs(waited),
            clauses.join("; ")
        ))
    }

    /// The last pass's longest part, for a test's look.
    #[cfg(test)]
    pub fn longest_last(&self) -> Option<(&'static str, Duration)> {
        let last = self.last.as_ref()?;
        last.parts.iter().copied().max_by_key(|(_, took)| *took)
    }

    /// `words` when a slow line is due, with the slow ones since the last line counted; else
    /// counted for the next.
    fn due(&mut self, now: Instant, words: String) -> Option<String> {
        if self
            .lined
            .is_some_and(|at| now.saturating_duration_since(at) < LINE_EVERY)
        {
            self.unlined += 1;
            return None;
        }
        self.lined = Some(now);
        Some(match std::mem::take(&mut self.unlined) {
            0 => words,
            1 => format!("{words} One more was slow since the line before."),
            more => format!("{words} {more} more were slow since the line before."),
        })
    }
}

/// `duration` in seconds with a tenth: `10.4 s`.
fn secs(duration: Duration) -> String {
    format!("{:.1} s", duration.as_secs_f64())
}

/// The longest parts, each name once with its time summed, as ` (the watch 9.8 s, the meter
/// 0.2 s)`; nothing when none reaches [`PART_FLOOR`].
fn named(parts: &[(&'static str, Duration)]) -> String {
    let mut summed: Vec<(&'static str, Duration)> = Vec::new();
    for (name, took) in parts {
        match summed.iter_mut().find(|(known, _)| known == name) {
            Some((_, total)) => *total += *took,
            None => summed.push((name, *took)),
        }
    }
    summed.retain(|(_, took)| *took >= PART_FLOOR);
    summed.sort_by_key(|(_, took)| std::cmp::Reverse(*took));
    if summed.is_empty() {
        return String::new();
    }
    let words: Vec<String> = summed
        .iter()
        .take(PARTS_NAMED)
        .map(|(name, took)| format!("{name} {}", secs(*took)))
        .collect();
    format!(" ({})", words.join(", "))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ms(millis: u64) -> Duration {
        Duration::from_millis(millis)
    }

    /// A pass that began at `start` with its parts, ended after them.
    fn pass(
        pace: &mut Pace,
        start: Instant,
        parts: &[(&'static str, u64)],
    ) -> (Instant, Option<String>) {
        pace.begin_at(start);
        let mut at = start;
        for (name, millis) in parts {
            at += ms(*millis);
            pace.mark_at(name, at);
        }
        (at, pace.end_at(at))
    }

    /// A form taken within the threshold adds nothing; one that waited says what the window did
    /// meanwhile: the pass before with its longest parts, the time between, the frame's drawing.
    #[test]
    fn a_long_wait_says_what_the_window_did_meanwhile() {
        let mut pace = Pace::default();
        let start = Instant::now();
        let (ended, line) = pass(
            &mut pace,
            start,
            &[
                ("the forms", 1),
                ("the watch", 9_800),
                ("the meter", 200),
                ("the pages", 10),
            ],
        );
        // That pass was slow: its own line names its longest parts, the short ones left out.
        assert_eq!(
            line.as_deref(),
            Some("The window's pass took 10.0 s (the watch 9.8 s, the meter 0.2 s).")
        );
        pace.drawn_at(ms(300), ended + ms(300));
        let next = ended + ms(900);
        pace.begin_at(next);
        let sent = start + ms(500);
        assert_eq!(
            pace.waited_at(next - ms(500), next),
            None,
            "within two seconds"
        );
        assert_eq!(
            pace.waited_at(sent, next).as_deref(),
            Some(
                "waited 10.4 s for the window: the pass before took 10.0 s (the watch 9.8 s, \
                 the meter 0.2 s); 0.9 s passed between it and this pass, 0.3 s of it the \
                 frame's drawing."
            )
        );
        // Taken after this pass's own parts: they are named too.
        pace.mark_at("the files", next + ms(1_200));
        let words = pace.waited_at(sent, next + ms(1_300)).unwrap();
        assert!(
            words.ends_with("; this pass took 1.3 s before the form (the files 1.2 s)."),
            "{words}"
        );
    }

    /// A hidden window draws no frame between its passes; the first pass has none before it.
    #[test]
    fn a_wait_without_a_frame_or_a_pass_before_says_so() {
        let mut pace = Pace::default();
        let start = Instant::now();
        pace.begin_at(start + ms(3_000));
        assert_eq!(
            pace.waited_at(start, start + ms(3_000)).as_deref(),
            Some("waited 3.0 s for the window: no pass before this one is on record.")
        );
        pace.end_at(start + ms(3_010));
        // A frame drawn before the pass before ended is not this gap's.
        pace.drawn_at(ms(100), start + ms(2_000));
        let next = start + ms(14_000);
        pace.begin_at(next);
        let words = pace.waited_at(start + ms(3_500), next).unwrap();
        assert!(
            words.contains("; 11.0 s passed between it and this pass, with no frame drawn."),
            "{words}"
        );
    }

    /// At most one slow line a minute, passes and frames together; the next one counts the rest.
    #[test]
    fn slow_lines_come_once_a_minute_with_the_others_counted() {
        let mut pace = Pace::default();
        let start = Instant::now();
        let (_, first) = pass(&mut pace, start, &[("the watch", 2_500)]);
        assert!(first.is_some());
        let (_, quick) = pass(&mut pace, start + ms(3_000), &[("the watch", 100)]);
        assert_eq!(quick, None, "a quick pass says nothing");
        let (_, held) = pass(&mut pace, start + ms(4_000), &[("the meter", 3_000)]);
        assert_eq!(held, None, "within the minute");
        assert_eq!(pace.drawn_at(ms(2_100), start + ms(8_000)), None);
        let (_, line) = pass(&mut pace, start + ms(61_000), &[("the pages", 2_000)]);
        assert_eq!(
            line.as_deref(),
            Some(
                "The window's pass took 2.0 s (the pages 2.0 s). 2 more were slow since the \
                 line before."
            )
        );
        let line = pace.drawn_at(ms(4_000), start + ms(125_000));
        assert_eq!(
            line.as_deref(),
            Some("The window's frame took 4.0 s to draw.")
        );
    }

    /// The same part twice in a pass (the forms at its start and after the notifications) is
    /// named once, its times summed; a mark outside a pass is nothing.
    #[test]
    fn a_part_named_twice_is_summed() {
        let mut pace = Pace::default();
        pace.mark("the forms");
        let start = Instant::now();
        let (_, line) = pass(
            &mut pace,
            start,
            &[
                ("the forms", 1_200),
                ("the watch", 600),
                ("the forms", 1_000),
            ],
        );
        assert_eq!(
            line.as_deref(),
            Some("The window's pass took 2.8 s (the forms 2.2 s, the watch 0.6 s).")
        );
    }
}
