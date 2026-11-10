/// A CSS `cubic-bezier()` curve from `(0, 0)` to `(1, 1)` through two control points, solved
/// for the output at an input progress as browsers solve it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct CubicBezier {
    x_coefficients: [f64; 3],
    y_coefficients: [f64; 3],
    /// The slope of the curve before its start and after its end, where it goes on as a line.
    start_gradient: f64,
    end_gradient: f64,
}

/// How close the curve is solved for the input progress.
const EPSILON: f64 = 1e-7;
const NEWTON_ITERATIONS: usize = 4;
const BISECTION_ITERATIONS: usize = 64;
/// A slope this flat stops the Newton steps: they would overshoot.
const FLAT_SLOPE: f64 = 1e-6;

impl CubicBezier {
    pub(crate) fn new(x1: f32, y1: f32, x2: f32, y2: f32) -> Self {
        let [x1, y1, x2, y2] = [x1, y1, x2, y2].map(f64::from);
        let is_zero = |coordinate: f64| coordinate.abs() < f64::EPSILON;
        let is_one = |coordinate: f64| (coordinate - 1.0).abs() < f64::EPSILON;
        let coefficients = |first: f64, second: f64| {
            let c = 3.0 * first;
            let b = 3.0 * (second - first) - c;
            let a = 1.0 - c - b;
            [a, b, c]
        };
        let start_gradient = if x1 > 0.0 {
            y1 / x1
        } else if is_zero(y1) && x2 > 0.0 {
            y2 / x2
        } else if is_zero(y1) && is_zero(y2) {
            1.0
        } else {
            0.0
        };
        let end_gradient = if x2 < 1.0 {
            (y2 - 1.0) / (x2 - 1.0)
        } else if is_one(y2) && x1 < 1.0 {
            (y1 - 1.0) / (x1 - 1.0)
        } else if is_one(y2) && is_one(y1) {
            1.0
        } else {
            0.0
        };
        Self {
            x_coefficients: coefficients(x1, x2),
            y_coefficients: coefficients(y1, y2),
            start_gradient,
            end_gradient,
        }
    }

    /// The output progress at `x`; outside `[0, 1]` the curve goes on along its end slopes.
    pub(crate) fn solve(&self, x: f64) -> f64 {
        if x < 0.0 {
            return self.start_gradient * x;
        }
        if x > 1.0 {
            return 1.0 + self.end_gradient * (x - 1.0);
        }
        sample(self.y_coefficients, self.parameter_at(x))
    }

    /// The curve parameter whose `x` is the given one.
    fn parameter_at(&self, x: f64) -> f64 {
        let mut parameter = x;
        for _ in 0..NEWTON_ITERATIONS {
            let error = sample(self.x_coefficients, parameter) - x;
            if error.abs() < EPSILON {
                return parameter;
            }
            let slope = slope(self.x_coefficients, parameter);
            if slope.abs() < FLAT_SLOPE {
                break;
            }
            parameter -= error / slope;
        }
        let (mut low, mut high) = (0.0, 1.0);
        parameter = x;
        for _ in 0..BISECTION_ITERATIONS {
            let sampled = sample(self.x_coefficients, parameter);
            if (sampled - x).abs() < EPSILON {
                break;
            }
            if x > sampled {
                low = parameter;
            } else {
                high = parameter;
            }
            parameter = (high - low) * 0.5 + low;
        }
        parameter
    }
}

fn sample([a, b, c]: [f64; 3], parameter: f64) -> f64 {
    ((a * parameter + b) * parameter + c) * parameter
}

fn slope([a, b, c]: [f64; 3], parameter: f64) -> f64 {
    (3.0 * a * parameter + 2.0 * b) * parameter + c
}
