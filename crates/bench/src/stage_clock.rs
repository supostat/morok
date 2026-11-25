use std::cmp::Ordering;
use std::time::{Duration, Instant};

use morok::components::view::SurfaceStage;
use morok::core::{FrameStage, StageObserver};

use crate::counting_allocator::CountingAllocator;
use crate::frame_sample::FrameSample;
use crate::stage::Stage;

/// Times a frame and counts its allocations by budget stage. It starts before the input or the
/// change of the frame; at the end of every stage of a run it puts what passed since the last
/// boundary into the budget stage of that stage's position, and the encoding, which the bench
/// runs after the run, closes the frame.
#[derive(Debug)]
pub(crate) struct StageClock {
    last_boundary: Instant,
    allocations_at_last_boundary: u64,
    stage_times: [Duration; Stage::ALL.len()],
    stage_allocations: [u64; Stage::ALL.len()],
}

impl StageClock {
    pub(crate) fn start() -> Self {
        Self {
            last_boundary: Instant::now(),
            allocations_at_last_boundary: CountingAllocator::allocations_so_far(),
            stage_times: [Duration::ZERO; Stage::ALL.len()],
            stage_allocations: [0; Stage::ALL.len()],
        }
    }

    /// Closes the encoding of the frame and gives what each budget stage of it cost.
    pub(crate) fn encoding_ended(mut self) -> FrameSample {
        self.close(Stage::Encoding);
        FrameSample::new(self.stage_times, self.stage_allocations)
    }

    fn close(&mut self, stage: Stage) {
        let now = Instant::now();
        let allocations = CountingAllocator::allocations_so_far();
        self.stage_times[stage.index()] += now - self.last_boundary;
        self.stage_allocations[stage.index()] += allocations - self.allocations_at_last_boundary;
        self.last_boundary = now;
        self.allocations_at_last_boundary = allocations;
    }
}

impl StageObserver<FrameStage> for StageClock {
    fn stage_ended(&mut self, stage: FrameStage) {
        self.close(budget_stage_of_tree_stage(stage));
    }
}

impl StageObserver<SurfaceStage> for StageClock {
    fn stage_ended(&mut self, stage: SurfaceStage) {
        self.close(budget_stage_of_surface_stage(stage));
    }
}

fn budget_stage_of_tree_stage(stage: FrameStage) -> Stage {
    budget_stage_by_position(stage, FrameStage::Layout)
}

fn budget_stage_of_surface_stage(stage: SurfaceStage) -> Stage {
    budget_stage_by_position(stage, SurfaceStage::Tree(FrameStage::Layout))
}

/// The stages a run declares before its layout prepare the frame for it, so they count as its
/// events; a stage added to a run lands in the budget stage of where it runs.
fn budget_stage_by_position<RunStage: Ord + Copy>(stage: RunStage, layout: RunStage) -> Stage {
    match stage.cmp(&layout) {
        Ordering::Less => Stage::Events,
        Ordering::Equal => Stage::Layout,
        Ordering::Greater => Stage::DisplayList,
    }
}

#[cfg(test)]
mod tests {
    use morok::components::view::SurfaceStage;
    use morok::core::FrameStage;

    use super::{budget_stage_of_surface_stage, budget_stage_of_tree_stage};
    use crate::stage::Stage;

    #[test]
    fn every_stage_of_the_run_of_a_tree_counts_into_the_budget_stage_of_its_position() {
        for (stage, budget_stage) in [
            (FrameStage::Styles, Stage::Events),
            (FrameStage::Images, Stage::Events),
            (FrameStage::Layout, Stage::Layout),
            (FrameStage::DisplayList, Stage::DisplayList),
        ] {
            assert_eq!(budget_stage_of_tree_stage(stage), budget_stage, "{stage:?}");
        }
    }

    #[test]
    fn every_stage_of_the_run_of_a_surface_counts_into_the_budget_stage_of_its_position() {
        for (stage, budget_stage) in [
            (SurfaceStage::Hover, Stage::Events),
            (SurfaceStage::Scrolls, Stage::Events),
            (SurfaceStage::Timers, Stage::Events),
            (SurfaceStage::Effects, Stage::Events),
            (SurfaceStage::Tree(FrameStage::Styles), Stage::Events),
            (SurfaceStage::Tree(FrameStage::Images), Stage::Events),
            (SurfaceStage::Tree(FrameStage::Layout), Stage::Layout),
            (SurfaceStage::Tree(FrameStage::DisplayList), Stage::DisplayList),
        ] {
            assert_eq!(budget_stage_of_surface_stage(stage), budget_stage, "{stage:?}");
        }
    }
}
