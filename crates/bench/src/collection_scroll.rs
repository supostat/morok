use morok::core::{Point, ScrollInput, ScrollPhase, Size};
use morok_catalog::styles;
use morok_platform::Renderer;

use crate::frame_budget::FrameBudget;
use crate::frame_measurements::FrameMeasurements;
use crate::frame_sample::FrameSample;
use crate::scrolled_collection::{CollectionParts, ScrolledCollection, row_name_in_slot};
use crate::surface_pipeline::SurfacePipeline;

const VIEWPORT: Size = Size {
    width: 800.0,
    height: 600.0,
};

/// The page of a collection of the catalog on a surface that is not on screen: the wheel
/// scrolls the collection from its first row to its last, at one speed, in as many frames as a
/// scenario runs. Every frame shows rows that the frame before did not, so every row slot is
/// bound to another row and sets another text.
#[derive(Debug)]
pub struct CollectionScroll {
    pipeline: SurfacePipeline,
    collection: ScrolledCollection,
    parts: CollectionParts,
    /// Where the wheel turns: over the middle of the collection.
    wheel_position: Point,
}

impl CollectionScroll {
    /// Mounts the page of the collection under the light theme of the catalog, on a surface
    /// whose icons the renderer draws, and runs its first frames: a collection the wheel
    /// scrolls is on screen already.
    pub fn new(renderer: &Renderer, collection: ScrolledCollection) -> Self {
        let page = collection.page();
        let pipeline = SurfacePipeline::new(renderer, styles(), VIEWPORT, None, |mount| page.mount(mount));
        let parts = collection.parts(pipeline.tree());
        let shown_in = pipeline.tree().surface_box(parts.viewport);
        Self {
            pipeline,
            collection,
            parts,
            wheel_position: Point {
                x: shown_in.x + shown_in.width / 2.0,
                y: shown_in.y + shown_in.height / 2.0,
            },
        }
    }

    /// Scrolls through the rows in the frames of the budget, the warm-up among them, and
    /// measures the frames after the warm-up.
    pub fn measure(&mut self, budget: &FrameBudget) -> FrameMeasurements {
        let frames = budget.warm_up_frames + budget.measured_frames;
        let scroll_per_frame = self.collection.row_count() as f32 * self.collection.row_height() / frames as f32;
        for frame in 1..=budget.warm_up_frames {
            self.run_frame(frame, scroll_per_frame);
        }
        let mut samples = Vec::with_capacity(budget.measured_frames);
        for frame in budget.warm_up_frames + 1..=frames {
            samples.push(self.run_frame(frame, scroll_per_frame));
        }
        FrameMeasurements::new(samples)
    }

    /// How far below the bottom of the viewport of the collection the slot that shows the last
    /// row ends on the surface, after the frames run so far; none while no slot shows the last
    /// row.
    pub fn last_row_below_viewport(&self) -> Option<f32> {
        let tree = self.pipeline.tree();
        let last_row_name = self.collection.last_row_name();
        tree.children(self.parts.slots)
            .iter()
            .find(|slot| row_name_in_slot(tree, **slot) == last_row_name)
            .map(|slot| tree.surface_box(*slot).bottom() - tree.surface_box(self.parts.viewport).bottom())
    }

    /// Runs one frame in which the wheel scrolls the collection down by `scroll`.
    fn run_frame(&mut self, frame: usize, scroll: f32) -> FrameSample {
        let input = ScrollInput {
            position: self.wheel_position,
            delta: Point { x: 0.0, y: scroll },
            phase: ScrollPhase::Discrete,
        };
        self.pipeline.run_frame(frame, |surface| surface.scroll_input(&input))
    }
}
