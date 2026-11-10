/// Whether a keyframe animation shows its keyframes before it starts and after it ends.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum FillMode {
    None,
    /// After the end the last keyframe it reached stays.
    Forwards,
    /// During the delay the keyframe it starts from shows.
    Backwards,
    Both,
}
