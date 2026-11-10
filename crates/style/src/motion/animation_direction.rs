/// Which way the iterations of a keyframe animation run through its keyframes.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum AnimationDirection {
    Normal,
    Reverse,
    /// Forwards on the first iteration, backwards on the second, and so on.
    Alternate,
    /// Backwards on the first iteration, forwards on the second, and so on.
    AlternateReverse,
}
