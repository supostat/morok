/// How many times a keyframe animation runs through its keyframes.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum IterationCount {
    /// Not negative; a fraction stops part way through the last iteration.
    Count(f32),
    Infinite,
}
