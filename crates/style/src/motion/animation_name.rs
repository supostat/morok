use crate::motion::keyframes::Keyframes;

/// One entry of `animation-name`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum AnimationName {
    /// The entry runs no animation.
    None,
    Keyframes(Keyframes),
}
