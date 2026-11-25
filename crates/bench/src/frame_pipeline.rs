use std::time::Duration;

use morok_core::{Size, TreeFrame};
use morok_platform::{CoreTextShaper, OffscreenTarget, Renderer};
use morok_style::ColorScheme;

use crate::bench_error::BenchError;
use crate::frame_budget::FrameBudget;
use crate::frame_measurements::FrameMeasurements;
use crate::frame_sample::FrameSample;
use crate::scene::Scene;
use crate::scene_tree::SceneTree;
use crate::stage_clock::StageClock;

/// Frames are measured as a Retina display shows them.
const DEVICE_SCALE: f32 = 2.0;
/// The frames follow one another at the rate of a `ProMotion` display.
const FRAMES_PER_SECOND: f64 = 120.0;

/// The frame of the kit on a surface that is not on screen: the run of the tree of a scene,
/// its pictures sized by the renderer, and the CPU time and heap allocations of each stage
/// taken frame by frame.
#[derive(Debug)]
pub struct FramePipeline {
    scene_tree: SceneTree,
    viewport: Size,
    text_shaper: CoreTextShaper,
    renderer: Renderer,
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
            renderer: renderer.clone(),
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

    /// Runs one frame: the changes of the scene and the run of its tree, then the encoding,
    /// timed by budget stage. The GPU draws the frame after the sample is taken.
    pub fn run_frame(&mut self, scene: &mut dyn Scene, frame: usize) -> FrameSample {
        let mut clock = StageClock::start();
        let tree = self.scene_tree.tree_mut();
        scene.change(frame, tree);
        let tree_frame = TreeFrame {
            viewport: self.viewport,
            scale: DEVICE_SCALE,
            now: Duration::from_secs_f64(frame as f64 / FRAMES_PER_SECOND),
            text_shaper: &mut self.text_shaper,
            images: &self.renderer,
        };
        tree.run_frame(tree_frame, &mut clock);
        self.target.draw(tree.display_list(), tree.theme());
        let sample = clock.encoding_ended();
        self.target.wait_until_drawn();
        sample
    }
}
