use morok::core::{ElementId, ElementTree, Point, ScrollInput, ScrollPhase, Size};
use morok_catalog::styles::CLASSES;
use morok_catalog::{Page, TABLE_ROWS, styles};
use morok_platform::Renderer;

use crate::frame_budget::FrameBudget;
use crate::frame_measurements::FrameMeasurements;
use crate::frame_sample::FrameSample;
use crate::surface_pipeline::SurfacePipeline;

/// How high the table of the kit makes a row.
const ROW_HEIGHT: f32 = 26.0;
const VIEWPORT: Size = Size {
    width: 800.0,
    height: 600.0,
};

/// The page of the table of the catalog, a table of a million rows, on a surface that is not on
/// screen: the wheel scrolls it from its first row to its last, at one speed, in as many frames
/// as a scenario runs. Every frame shows rows that the frame before did not, so every row slot
/// is bound to another row and every cell sets another text.
#[derive(Debug)]
pub struct MillionRows {
    pipeline: SurfacePipeline,
    /// Where the wheel turns: over the middle of the table.
    wheel_position: Point,
}

impl MillionRows {
    /// Mounts the page under the light theme of the catalog, on a surface whose icons the
    /// renderer draws, and runs its first frames: a table the wheel scrolls is on screen
    /// already.
    pub fn new(renderer: &Renderer) -> Self {
        let pipeline = SurfacePipeline::new(renderer, styles(), VIEWPORT, None, |mount| Page::Table.mount(mount));
        let table = table_of(pipeline.tree());
        let shown_in = pipeline.tree().surface_box(table);
        Self {
            pipeline,
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
        let scroll_per_frame = TABLE_ROWS as f32 * ROW_HEIGHT / frames as f32;
        for frame in 1..=budget.warm_up_frames {
            self.run_frame(frame, scroll_per_frame);
        }
        let mut samples = Vec::with_capacity(budget.measured_frames);
        for frame in budget.warm_up_frames + 1..=frames {
            samples.push(self.run_frame(frame, scroll_per_frame));
        }
        FrameMeasurements::new(samples)
    }

    /// How far below the bottom of the viewport of the table the slot that shows the last row
    /// ends on the surface, after the frames run so far; none while no slot shows the last row.
    pub fn last_row_below_viewport(&self) -> Option<f32> {
        let tree = self.pipeline.tree();
        let table = table_of(tree);
        let viewport = tree.children(table)[0];
        let content = tree.children(viewport)[0];
        let rows = tree.children(content)[0];
        let last_row_name = format!("Document {TABLE_ROWS}");
        tree.children(rows)
            .iter()
            .find(|slot| {
                let row = tree.children(**slot)[0];
                tree.text(tree.children(row)[0]) == last_row_name
            })
            .map(|slot| tree.surface_box(*slot).bottom() - tree.surface_box(viewport).bottom())
    }

    /// Runs one frame in which the wheel scrolls the table down by `scroll`.
    fn run_frame(&mut self, frame: usize, scroll: f32) -> FrameSample {
        let input = ScrollInput {
            position: self.wheel_position,
            delta: Point { x: 0.0, y: scroll },
            phase: ScrollPhase::Discrete,
        };
        self.pipeline.run_frame(frame, |surface| surface.scroll_input(&input))
    }
}

/// The table of the page.
fn table_of(tree: &ElementTree) -> ElementId {
    let mut unvisited = vec![tree.root()];
    while let Some(element) = unvisited.pop() {
        if tree.classes(element).contains(&CLASSES.table) {
            return element;
        }
        unvisited.extend(tree.children(element));
    }
    panic!("the page of the table shows a table")
}
