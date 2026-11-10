use std::time::{Duration, Instant};

use morok_core::Size;
use morok_platform::{CoreTextShaper, OffscreenTarget, Renderer};
use morok_style::ColorScheme;

use crate::bench_error::BenchError;
use crate::counting_allocator::CountingAllocator;
use crate::frame_budget::FrameBudget;
use crate::frame_measurements::FrameMeasurements;
use crate::frame_sample::FrameSample;
use crate::scene::Scene;
use crate::scene_tree::SceneTree;

/// Frames are measured as a Retina display shows them.
const DEVICE_SCALE: f32 = 2.0;
/// The frames follow one another at the rate of a `ProMotion` display.
const FRAMES_PER_SECOND: f64 = 120.0;

/// The frame pipeline of the kit on a surface that is not on screen: the stages run on the
/// tree of a scene and their CPU time and heap allocations are taken frame by frame.
#[derive(Debug)]
pub struct FramePipeline {
    scene_tree: SceneTree,
    viewport: Size,
    text_shaper: CoreTextShaper,
    target: OffscreenTarget,
}

impl FramePipeline {
    /// Builds the tree of the scene under the light theme and the surface it is drawn on.
    pub fn new(renderer: &Renderer, scene: &mut dyn Scene) -> Result<Self, BenchError> {
        let viewport = scene.viewport();
        Ok(Self {
            scene_tree: SceneTree::of(scene, ColorScheme::Light)?,
            viewport,
            text_shaper: CoreTextShaper::new(),
            target: OffscreenTarget::new(renderer, viewport, DEVICE_SCALE),
        })
    }

    /// Runs the warm-up frames of the budget unmeasured, then measures as many frames as the
    /// budget asks for.
    pub fn measure(&mut self, scene: &mut dyn Scene, budget: &FrameBudget) -> FrameMeasurements {
        for frame in 0..budget.warm_up_frames {
            self.run_frame(scene, frame);
        }
        let mut samples = Vec::with_capacity(budget.measured_frames);
        for frame in budget.warm_up_frames..budget.warm_up_frames + budget.measured_frames {
            samples.push(self.run_frame(scene, frame));
        }
        FrameMeasurements::new(samples)
    }

    /// Runs one frame: the changes of the scene and the stages of the pipeline, timed one
    /// after another. The GPU draws the frame after the sample is taken.
    pub fn run_frame(&mut self, scene: &mut dyn Scene, frame: usize) -> FrameSample {
        let allocations_before = CountingAllocator::allocations_so_far();
        let tree = self.scene_tree.tree_mut();
        let started = Instant::now();
        scene.change(frame, tree);
        tree.resolve_styles(Duration::from_secs_f64(frame as f64 / FRAMES_PER_SECOND));
        let events_done = Instant::now();
        tree.compute_layout(self.viewport, DEVICE_SCALE, &mut self.text_shaper);
        let layout_done = Instant::now();
        tree.build_display_list();
        let display_list_done = Instant::now();
        self.target.draw(tree.display_list());
        let encoding_done = Instant::now();
        let allocations = CountingAllocator::allocations_so_far() - allocations_before;
        self.target.wait_until_drawn();
        FrameSample::new(
            [
                events_done - started,
                layout_done - events_done,
                display_list_done - layout_done,
                encoding_done - display_list_done,
            ],
            allocations,
        )
    }
}
