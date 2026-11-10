use std::ops::RangeInclusive;

use morok_core::NumericValue;

/// How far off a whole number of steps a range may come out and still be taken for one.
const STEP_COUNT_TOLERANCE: f64 = 1e-9;

/// The values a slider moves through: from `min` to `max` in steps of `step` from `min`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct SliderScale {
    min: f64,
    max: f64,
    step: f64,
    /// How many steps lie between `min` and the highest value of the scale, which is `max`
    /// when `max` is on a step and the last step before it when it is not.
    steps_to_highest: f64,
}

impl SliderScale {
    /// # Panics
    ///
    /// When the range holds one value or none, or the step does not move forward.
    pub(crate) fn new(range: RangeInclusive<f64>, step: f64) -> Self {
        let (min, max) = range.into_inner();
        assert!(min < max, "a slider moves through more than one value");
        assert!(step > 0.0 && step.is_finite(), "a slider moves in steps forward");
        let steps_in_range = (max - min) / step;
        let rounded = steps_in_range.round();
        // A range that is a whole number of steps comes out a hair off it in floating point.
        let steps_to_highest = if (steps_in_range - rounded).abs() < STEP_COUNT_TOLERANCE {
            rounded
        } else {
            steps_in_range.floor()
        };
        Self {
            min,
            max,
            step,
            steps_to_highest,
        }
    }

    /// The value of the scale nearest to `value`: on a step, between the lowest and the
    /// highest value.
    pub(crate) fn snapped(self, value: f64) -> f64 {
        let steps = ((value - self.min) / self.step)
            .round()
            .clamp(0.0, self.steps_to_highest);
        self.min + steps * self.step
    }

    /// The value `steps` steps away from `value`; a negative count goes down.
    pub(crate) fn stepped(self, value: f64, steps: f64) -> f64 {
        self.snapped(value + steps * self.step)
    }

    /// The value a tenth of the way along the scale from `value`, or a step when a step is
    /// longer; a negative count goes down.
    pub(crate) fn paged(self, value: f64, pages: f64) -> f64 {
        let page = self.step.max((self.max - self.min) / 10.0);
        self.snapped(value + pages * page)
    }

    pub(crate) fn lowest(self) -> f64 {
        self.min
    }

    pub(crate) fn highest(self) -> f64 {
        self.min + self.steps_to_highest * self.step
    }

    /// How far along the scale `value` lies: zero at `min`, one at `max`.
    pub(crate) fn fraction_of(self, value: f64) -> f64 {
        ((value - self.min) / (self.max - self.min)).clamp(0.0, 1.0)
    }

    /// The value of the scale nearest to the place `fraction` of the way along it.
    pub(crate) fn value_at(self, fraction: f64) -> f64 {
        self.snapped(self.min + fraction.clamp(0.0, 1.0) * (self.max - self.min))
    }

    /// `value` as an assistive technology reads it.
    pub(crate) fn numeric_value(self, value: f64) -> NumericValue {
        NumericValue {
            value,
            min: self.min,
            max: self.max,
            step: self.step,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::SliderScale;

    fn assert_close(actual: f64, expected: f64) {
        assert!((actual - expected).abs() < 1e-12, "{actual} is not {expected}");
    }

    #[test]
    fn a_value_snaps_to_the_nearest_step_from_the_minimum_between_the_ends() {
        let scale = SliderScale::new(10.0..=60.0, 20.0);

        assert_close(scale.snapped(19.0), 10.0);
        assert_close(scale.snapped(21.0), 30.0);
        assert_close(scale.snapped(-5.0), 10.0);
    }

    #[test]
    fn a_maximum_that_lies_on_no_step_is_never_reached() {
        let scale = SliderScale::new(10.0..=60.0, 20.0);

        assert_close(scale.snapped(59.0), 50.0);
        assert_close(scale.snapped(500.0), 50.0);
        assert_close(scale.highest(), 50.0);
    }

    #[test]
    fn a_page_is_a_tenth_of_the_scale_unless_a_step_is_longer() {
        assert_close(SliderScale::new(0.0..=100.0, 1.0).paged(50.0, 1.0), 60.0);
        assert_close(SliderScale::new(0.0..=100.0, 25.0).paged(50.0, -1.0), 25.0);
    }

    #[test]
    fn a_place_along_the_scale_maps_to_the_nearest_value() {
        let scale = SliderScale::new(0.0..=1.0, 0.25);

        assert_close(scale.value_at(0.3), 0.25);
        assert_close(scale.value_at(1.4), 1.0);
        assert_close(scale.fraction_of(0.75), 0.75);
    }

    #[test]
    fn a_range_of_a_whole_number_of_steps_reaches_its_end() {
        assert_close(SliderScale::new(0.0..=1.0, 0.1).highest(), 1.0);
        assert_close(SliderScale::new(0.0..=1.0, 0.3).highest(), 0.9);
    }
}
