use serde::Deserialize;

use crate::milliseconds::Milliseconds;
use crate::stage::Stage;

/// How long the 95th percentile of each stage may take.
#[derive(Clone, Copy, Debug, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StageBudgets {
    pub events: Milliseconds,
    pub layout: Milliseconds,
    pub display_list: Milliseconds,
    pub encoding: Milliseconds,
}

impl StageBudgets {
    pub fn of(&self, stage: Stage) -> Milliseconds {
        match stage {
            Stage::Events => self.events,
            Stage::Layout => self.layout,
            Stage::DisplayList => self.display_list,
            Stage::Encoding => self.encoding,
        }
    }
}
