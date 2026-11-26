/// Hears the end of every stage of a frame as the frame runs: what measures a frame stage by
/// stage without knowing the order of the stages.
pub trait StageObserver<Stage> {
    /// Hears that `stage` of the frame has just ended.
    fn stage_ended(&mut self, stage: Stage);
}

/// Hears the end of every stage and does nothing: the observer of a frame nobody measures.
#[derive(Clone, Copy, Debug, Default)]
pub struct Unobserved;

impl<Stage> StageObserver<Stage> for Unobserved {
    fn stage_ended(&mut self, _stage: Stage) {}
}
