use morok_platform::Renderer;

use crate::budgets::Budgets;
use crate::frame_gate;
use crate::million_rows::MillionRows;
use crate::scroll_gate;
use crate::verdict::Verdict;

/// Scrolls a table of a million rows from its first row to its last through the pipeline of the
/// kit and holds the frames of the scroll against the frame budgets, and the place the scroll
/// ended at against the last row.
pub fn run(scenario: &str, renderer: &Renderer, budgets: &Budgets) -> Verdict {
    let mut rows = MillionRows::new(renderer);
    let measurements = rows.measure(&budgets.frame);
    let mut verdict = frame_gate::judge(scenario, &measurements, budgets);
    verdict.check(scroll_gate::judge_end(rows.last_row_below_viewport()));
    verdict
}
