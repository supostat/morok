use morok_core::{FrameStage, StageObserver};

use crate::view::surface_stage::SurfaceStage;

/// Hands the stages of the tree of a surface to the observer of the frame of the surface.
pub(crate) struct TreeStageRelay<'a> {
    pub(crate) observer: &'a mut dyn StageObserver<SurfaceStage>,
}

impl StageObserver<FrameStage> for TreeStageRelay<'_> {
    fn stage_ended(&mut self, stage: FrameStage) {
        self.observer.stage_ended(SurfaceStage::Tree(stage));
    }
}
