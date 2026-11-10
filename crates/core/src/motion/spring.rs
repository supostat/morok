use std::time::Duration;

use crate::motion::spring_sample::SpringSample;

/// A critically damped spring with a unit mass: the value returns to its target as fast as it
/// can without overshooting it, from wherever it starts and however fast it moves.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Spring {
    /// The pull towards the target per unit of distance; the stiffer the spring, the sooner the
    /// value arrives.
    pub stiffness: f32,
    /// The distance from the target, and the speed per second, under which the value is at rest.
    pub precision: f32,
}

impl Spring {
    /// How long a spring under reduced motion takes to fade its value in at the target.
    pub const REDUCED_MOTION_FADE: Duration = Duration::from_millis(150);

    /// Where a value moving towards `to` stands `elapsed` after it was at `from` with `velocity`
    /// units per second.
    pub fn sample(&self, from: f32, to: f32, velocity: f32, elapsed: Duration) -> SpringSample {
        let natural_frequency = f64::from(self.stiffness).sqrt();
        let seconds = elapsed.as_secs_f64();
        let displacement = f64::from(from - to);
        let rate = f64::from(velocity) + natural_frequency * displacement;
        let decay = (-natural_frequency * seconds).exp();
        let offset = (displacement + rate * seconds) * decay;
        let speed = (f64::from(velocity) - natural_frequency * rate * seconds) * decay;
        let precision = f64::from(self.precision);
        if offset.abs() < precision && speed.abs() < precision {
            return SpringSample::at_rest(to);
        }
        SpringSample {
            value: to + offset as f32,
            velocity: speed as f32,
            opacity: 1.0,
            is_settled: false,
        }
    }

    /// What a spring shows under reduced motion `elapsed` after it starts: the value is at its
    /// target at once and fades in there.
    pub fn fade(to: f32, elapsed: Duration) -> SpringSample {
        if elapsed >= Self::REDUCED_MOTION_FADE {
            return SpringSample::at_rest(to);
        }
        SpringSample {
            value: to,
            velocity: 0.0,
            opacity: (elapsed.as_secs_f64() / Self::REDUCED_MOTION_FADE.as_secs_f64()) as f32,
            is_settled: false,
        }
    }
}
