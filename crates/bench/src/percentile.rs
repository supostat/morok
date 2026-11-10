use std::time::Duration;

const PERCENTILE: f64 = 0.95;

/// The value at the 95th percentile by the nearest-rank method: the smallest value that is at
/// least as large as 95 % of them.
///
/// # Panics
///
/// When there is no value: a percentile of nothing names none.
pub(crate) fn percentile_95(mut values: Vec<Duration>) -> Duration {
    values.sort_unstable();
    let rank = (PERCENTILE * values.len() as f64).ceil() as usize;
    values[rank.max(1) - 1]
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::percentile_95;

    fn milliseconds(values: &[u64]) -> Vec<Duration> {
        values.iter().map(|value| Duration::from_millis(*value)).collect()
    }

    #[test]
    fn the_percentile_of_one_value_is_that_value() {
        assert_eq!(percentile_95(milliseconds(&[7])), Duration::from_millis(7));
    }

    #[test]
    fn the_percentile_of_twenty_values_is_the_nineteenth_in_order() {
        let values: Vec<u64> = (1..=20).rev().collect();
        assert_eq!(percentile_95(milliseconds(&values)), Duration::from_millis(19));
    }

    #[test]
    fn one_slow_frame_in_a_hundred_does_not_reach_the_percentile() {
        let mut values = vec![1; 99];
        values.push(50);
        assert_eq!(percentile_95(milliseconds(&values)), Duration::from_millis(1));
    }

    #[test]
    fn six_slow_frames_in_a_hundred_do() {
        let mut values = vec![1; 94];
        values.extend([50; 6]);
        assert_eq!(percentile_95(milliseconds(&values)), Duration::from_millis(50));
    }
}
