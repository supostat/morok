use crate::corner::Corner;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum GradientDirection {
    /// Degrees clockwise from the upward direction: 0 points up and 90 to the right.
    Angle(f32),
    /// Towards a corner; the angle depends on the proportions of the box.
    ToCorner(Corner),
}
