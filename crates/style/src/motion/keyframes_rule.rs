use std::borrow::Cow;

use crate::motion::keyframe::Keyframe;

/// A `@keyframes` rule.
#[derive(Clone, Debug, PartialEq)]
pub struct KeyframesRule {
    pub name: Cow<'static, str>,
    /// Ordered by offset, one keyframe per offset.
    pub keyframes: Cow<'static, [Keyframe]>,
}
