use std::borrow::Cow;

use crate::declaration::Declaration;

/// The declarations a keyframe animation reaches at one point of an iteration.
#[derive(Clone, Debug, PartialEq)]
pub struct Keyframe {
    /// From `0.0`, the start of an iteration, to `1.0`, its end.
    pub offset: f32,
    /// Animatable declarations in source order: a later declaration of a property overrides an
    /// earlier one.
    pub declarations: Cow<'static, [Declaration]>,
}
