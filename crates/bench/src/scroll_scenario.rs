use morok_platform::Renderer;

use crate::budgets::Budgets;
use crate::collection_scroll::CollectionScroll;
use crate::frame_gate;
use crate::scroll_gate;
use crate::scrolled_collection::ScrolledCollection;
use crate::verdict::Verdict;

/// Scrolls a collection of the catalog from its first row to its last through the pipeline of
/// the kit and holds the frames of the scroll against the frame budgets, and the place the
/// scroll ended at against the last row.
pub fn run(scenario: &str, renderer: &Renderer, budgets: &Budgets, collection: ScrolledCollection) -> Verdict {
    let mut scroll = CollectionScroll::new(renderer, collection);
    let measurements = scroll.measure(&budgets.frame);
    let mut verdict = frame_gate::judge(scenario, &measurements, budgets);
    verdict.check(scroll_gate::judge_end(scroll.last_row_below_viewport()));
    verdict
}
