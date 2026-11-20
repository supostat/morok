use std::time::{Duration, Instant};

use morok_catalog::{Catalog, Page};

use crate::bench_error::BenchError;
use crate::idle_budget::IdleBudget;
use crate::idle_gate;
use crate::idle_observation::IdleObservation;
use crate::verdict::Verdict;

/// How long the window gets to come on screen and present its first frame.
const FIRST_FRAME_TIMEOUT: Duration = Duration::from_secs(5);
/// The run loop is given this long at a time while the first frame is waited for.
const FIRST_FRAME_POLL: Duration = Duration::from_millis(100);
/// After the first frame, the time the window gets to finish opening before it counts as idle.
const SETTLING: Duration = Duration::from_secs(1);

/// Launches the catalog on its first page, lets its window settle, leaves it without events for
/// the duration of the budget and holds what its frame loop did against the budget.
pub(crate) fn run(scenario: &str, budget: &IdleBudget) -> Result<Verdict, BenchError> {
    let catalog = Catalog::launch(Page::Buttons)?;
    let application = catalog.application();
    let statistics = || catalog.windows()[0].frame_statistics();
    let opened = Instant::now();
    while statistics().presented_frames == 0 {
        if opened.elapsed() >= FIRST_FRAME_TIMEOUT {
            return Err(BenchError::NoFirstFrame {
                waited: opened.elapsed(),
            });
        }
        application.run_for(FIRST_FRAME_POLL);
    }
    application.run_for(SETTLING);
    let settled = statistics();
    application.run_for(budget.duration());
    let after_idle = statistics();
    let observation = IdleObservation {
        settling_frames: settled.presented_frames,
        duration: budget.duration(),
        presented_frames: after_idle.presented_frames - settled.presented_frames,
        display_link_callbacks: after_idle.display_link_callbacks - settled.display_link_callbacks,
    };
    Ok(idle_gate::judge(scenario, &observation, budget))
}
