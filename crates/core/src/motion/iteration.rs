/// Where a motion stands in its iteration at some time.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Iteration {
    /// From `0.0` to `1.0`, in the direction the iteration runs, before any easing.
    pub(crate) progress: f64,
    /// Whether the motion has not started yet: a step easing does not take its first jump.
    pub(crate) is_before: bool,
}
