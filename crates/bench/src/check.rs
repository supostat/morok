/// One measured value of a scenario held against its budget.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Check {
    /// What was measured: a stage, the whole frame, the allocations, the presented frames.
    pub subject: &'static str,
    pub measured: String,
    pub budget: String,
    pub holds: bool,
}
