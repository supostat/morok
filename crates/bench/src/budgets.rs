use std::fs;
use std::path::Path;

use serde::Deserialize;

use crate::bench_error::BenchError;
use crate::frame_budget::FrameBudget;
use crate::idle_budget::IdleBudget;
use crate::stage_budgets::StageBudgets;

const BUDGETS_FILE: &str = "budgets.toml";

/// The budgets every scenario is gated by.
#[derive(Clone, Copy, Debug, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Budgets {
    pub stages: StageBudgets,
    pub frame: FrameBudget,
    pub idle: IdleBudget,
}

impl Budgets {
    /// The budgets committed next to this crate.
    pub fn of_workspace() -> Result<Self, BenchError> {
        Self::read(&Path::new(env!("CARGO_MANIFEST_DIR")).join(BUDGETS_FILE))
    }

    pub fn read(path: &Path) -> Result<Self, BenchError> {
        let text = fs::read_to_string(path).map_err(|error| BenchError::File {
            path: path.to_path_buf(),
            error,
        })?;
        toml::from_str(&text).map_err(|error| BenchError::Malformed {
            path: path.to_path_buf(),
            reason: error.to_string(),
        })
    }
}
