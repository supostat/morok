use crate::motion::animation::Animation;
use crate::motion::held_value::HeldValue;
use crate::motion::transform_step::TransformStep;

/// Space the motion of the elements is worked out in, kept from frame to frame so that a frame
/// in motion allocates nothing.
#[derive(Debug)]
pub(crate) struct MotionScratch {
    /// The value of a property worked out for the frame.
    pub(crate) value: HeldValue,
    /// The value a running transition shows when the style it runs to changes.
    pub(crate) current: HeldValue,
    /// The transforms of the two keyframes around the progress of an animation.
    pub(crate) from_steps: Vec<TransformStep>,
    pub(crate) to_steps: Vec<TransformStep>,
    /// The animations of an element as `animation-name` lists them now.
    pub(crate) animations: Vec<Animation>,
}

impl Default for MotionScratch {
    fn default() -> Self {
        Self {
            value: HeldValue::Opacity(1.0),
            current: HeldValue::Opacity(1.0),
            from_steps: Vec::new(),
            to_steps: Vec::new(),
            animations: Vec::new(),
        }
    }
}
