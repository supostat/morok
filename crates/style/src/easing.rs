use crate::step_position::StepPosition;

/// How the progress of a transition or an animation runs between its start and its end: the
/// timing function of CSS.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Easing {
    Linear,
    CubicBezier { x1: f32, y1: f32, x2: f32, y2: f32 },
    Steps { count: u32, position: StepPosition },
}
