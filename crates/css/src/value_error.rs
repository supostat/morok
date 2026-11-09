use crate::problem::Problem;

/// Why a value parser gave up.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum ValueError {
    /// The input is not what this parser reads; the property reports what it expects.
    Mismatch,
    /// The input is what this parser reads, and the subset rejects it for a reason of its own.
    Problem(Problem),
}

pub(crate) type ValueResult<T> = Result<T, ValueError>;

impl From<Problem> for ValueError {
    fn from(problem: Problem) -> Self {
        Self::Problem(problem)
    }
}
