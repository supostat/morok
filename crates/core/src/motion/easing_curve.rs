use morok_style::{Easing, StepPosition};

use crate::motion::cubic_bezier::CubicBezier;

/// The output progress of `easing` at the input `progress`. `before` tells a step easing that
/// the motion has not started yet, so that a jump at the start is not taken early.
pub(crate) fn eased(easing: &Easing, progress: f64, before: bool) -> f64 {
    match *easing {
        Easing::Linear => progress,
        Easing::CubicBezier { x1, y1, x2, y2 } => CubicBezier::new(x1, y1, x2, y2).solve(progress),
        Easing::Steps { count, position } => stepped(f64::from(count), position, progress, before),
    }
}

fn stepped(count: f64, position: StepPosition, progress: f64, before: bool) -> f64 {
    let mut step = (progress * count).floor();
    if matches!(position, StepPosition::JumpStart | StepPosition::JumpBoth) {
        step += 1.0;
    }
    if before && (progress * count).fract() == 0.0 {
        step -= 1.0;
    }
    if progress >= 0.0 && step < 0.0 {
        step = 0.0;
    }
    let jumps = match position {
        StepPosition::JumpStart | StepPosition::JumpEnd => count,
        StepPosition::JumpNone => count - 1.0,
        StepPosition::JumpBoth => count + 1.0,
    };
    if progress <= 1.0 && step > jumps {
        step = jumps;
    }
    step / jumps
}
