/// Where the value of a spring stands at one time.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SpringSample {
    pub value: f32,
    /// Units per second, towards the target or away from it.
    pub velocity: f32,
    /// From `0.0` to `1.0`: under reduced motion the spring fades its content in instead of
    /// moving it; otherwise `1.0`.
    pub opacity: f32,
    /// Whether the value has arrived and will not change any more.
    pub is_settled: bool,
}

impl SpringSample {
    pub(crate) const fn at_rest(target: f32) -> Self {
        Self {
            value: target,
            velocity: 0.0,
            opacity: 1.0,
            is_settled: true,
        }
    }
}
