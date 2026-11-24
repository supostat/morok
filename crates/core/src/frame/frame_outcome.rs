/// What the stages of a frame did, each counted as the stage counts it.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct FrameOutcome {
    pub styled_elements: usize,
    pub laid_out_elements: usize,
    /// The elements whose display items were built again.
    pub rebuilt_elements: usize,
}
