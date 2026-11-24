use morok::core::{ElementId, ElementTree};
use morok::style::States;
use morok_catalog::{DiskMap, Gallery, Page, WINDOW_SIZE, styles};
use morok_platform::Renderer;

use crate::frame_budget::FrameBudget;
use crate::frame_measurements::FrameMeasurements;
use crate::frame_sample::FrameSample;
use crate::surface_pipeline::SurfacePipeline;

/// The share of the elements of the screen that changes from one frame to the next, as on the
/// reference scene.
const CHANGED_SHARE: f64 = 0.01;

/// A screen of the catalog as large as a window of the catalog, on a surface that is not on
/// screen, its canvas drawn by the painter of the catalog: one per cent of its elements turn
/// hovered or stop being hovered every frame, round after round.
#[derive(Debug)]
pub struct ScreenFrames {
    pipeline: SurfacePipeline,
    elements: Vec<ElementId>,
    changes_per_frame: usize,
    next_change: usize,
}

impl ScreenFrames {
    /// Mounts the screen under the light theme of the catalog and runs its first frame.
    pub fn new(renderer: &Renderer, screen: Page) -> Self {
        let dropping = renderer.clone();
        let gallery = Gallery::new(
            |encoded| renderer.add_image(encoded),
            move |encoded| Some(dropping.add_image(encoded)),
        );
        let pipeline = SurfacePipeline::new(
            renderer,
            styles(),
            WINDOW_SIZE,
            Some(Box::new(DiskMap::painter())),
            |mount| screen.mount(mount, &gallery),
        );
        let elements = elements_of(pipeline.tree());
        let changes_per_frame = ((elements.len() as f64 * CHANGED_SHARE) as usize).max(1);
        Self {
            pipeline,
            elements,
            changes_per_frame,
            next_change: 0,
        }
    }

    /// Runs the warm-up frames of the budget unmeasured, then measures as many frames as the
    /// budget asks for.
    pub fn measure(&mut self, budget: &FrameBudget) -> FrameMeasurements {
        for frame in 1..=budget.warm_up_frames {
            self.run_frame(frame);
        }
        let mut samples = Vec::with_capacity(budget.measured_frames);
        for frame in budget.warm_up_frames + 1..=budget.warm_up_frames + budget.measured_frames {
            samples.push(self.run_frame(frame));
        }
        FrameMeasurements::new(samples)
    }

    /// Runs one frame, which flips the hover state of the next elements in order.
    fn run_frame(&mut self, frame: usize) -> FrameSample {
        let Self {
            pipeline,
            elements,
            changes_per_frame,
            next_change,
        } = self;
        pipeline.run_frame(frame, |surface| {
            let tree = surface.tree_mut();
            for _ in 0..*changes_per_frame {
                let element = elements[*next_change];
                let is_hovered = tree.states(element).contains(States::HOVER);
                tree.set_state(element, States::HOVER, !is_hovered);
                *next_change = (*next_change + 1) % elements.len();
            }
        })
    }
}

/// The elements of the tree in document order.
fn elements_of(tree: &ElementTree) -> Vec<ElementId> {
    let mut elements = Vec::new();
    let mut unvisited = vec![tree.root()];
    while let Some(element) = unvisited.pop() {
        elements.push(element);
        unvisited.extend(tree.children(element).iter().rev());
    }
    elements
}
