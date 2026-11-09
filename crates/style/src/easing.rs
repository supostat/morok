use crate::step_position::StepPosition;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Easing {
    Linear,
    CubicBezier { x1: f32, y1: f32, x2: f32, y2: f32 },
    Steps { count: u32, position: StepPosition },
}
