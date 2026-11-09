/// Where the jumps of a `steps()` easing fall.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum StepPosition {
    JumpStart,
    JumpEnd,
    JumpNone,
    JumpBoth,
}
