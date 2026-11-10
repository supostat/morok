use std::time::Duration;

/// How long the caret is shown, and then how long it is hidden.
const HALF_PERIOD: Duration = Duration::from_millis(530);

/// The blinking of a caret: it is shown from the moment it moves or its text changes, then
/// hidden and shown by turns.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct CaretBlink {
    /// The time of the first frame since the caret moved; none until that frame runs.
    shown_since: Option<Duration>,
}

impl CaretBlink {
    /// The caret moved: it is shown again for a whole half period from the next frame on.
    pub(crate) fn restart(&mut self) {
        self.shown_since = None;
    }

    /// Whether the frame at `now` shows the caret, and how long from then until it turns.
    pub(crate) fn phase_at(&mut self, now: Duration) -> (bool, Duration) {
        let shown_since = *self.shown_since.get_or_insert(now);
        let elapsed = now.saturating_sub(shown_since);
        let half_periods = elapsed.as_nanos() / HALF_PERIOD.as_nanos();
        let into_half_period = elapsed.as_nanos() % HALF_PERIOD.as_nanos();
        let until_turn = HALF_PERIOD.as_nanos() - into_half_period;
        let until_turn = Duration::from_nanos(u64::try_from(until_turn).expect("a half period fits in 64 bits"));
        (half_periods.is_multiple_of(2), until_turn)
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::CaretBlink;

    const fn milliseconds(count: u64) -> Duration {
        Duration::from_millis(count)
    }

    #[test]
    fn a_caret_is_shown_for_a_half_period_from_its_first_frame_and_then_turns() {
        let mut blink = CaretBlink::default();

        assert_eq!(blink.phase_at(milliseconds(1000)), (true, milliseconds(530)));
        assert_eq!(blink.phase_at(milliseconds(1500)), (true, milliseconds(30)));
        assert_eq!(blink.phase_at(milliseconds(1530)), (false, milliseconds(530)));
        assert_eq!(blink.phase_at(milliseconds(2060)), (true, milliseconds(530)));
    }

    #[test]
    fn a_caret_that_moves_is_shown_again_from_the_next_frame() {
        let mut blink = CaretBlink::default();
        blink.phase_at(milliseconds(0));
        assert!(!blink.phase_at(milliseconds(600)).0);

        blink.restart();

        assert_eq!(blink.phase_at(milliseconds(610)), (true, milliseconds(530)));
    }
}
