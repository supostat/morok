use std::borrow::Cow;
use std::fmt::Write;
use std::time::{Duration, Instant};

use morok::Styles;
use morok::components::view::{Mount, SurfaceId, Surfaces};
use morok::components::{CellAlignment, Table, TableColumn};
use morok::core::{ElementId, Point, ScrollInput, ScrollPhase, Size};
use morok::reactive::Runtime;
use morok::style::{Declaration, LengthPercentage, LengthPercentageAuto, Pixels};
use morok_platform::{CoreTextShaper, OffscreenTarget, Renderer};

use crate::counting_allocator::CountingAllocator;
use crate::frame_budget::FrameBudget;
use crate::frame_measurements::FrameMeasurements;
use crate::frame_sample::FrameSample;

const ROWS: usize = 1_000_000;
/// How high the table of the kit makes a row.
const ROW_HEIGHT: f32 = 26.0;
const VIEWPORT: Size = Size {
    width: 800.0,
    height: 600.0,
};
/// Frames are measured as a Retina display shows them.
const DEVICE_SCALE: f32 = 2.0;
/// The frames follow one another at the rate of a `ProMotion` display.
const FRAMES_PER_SECOND: f64 = 120.0;
const KINDS: [&str; 4] = ["folder", "document", "disk image", "model"];
const WRITTEN: &str = "a string takes what is written to it";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum FileColumn {
    Name,
    Size,
    Modified,
    Kind,
}

/// A table of the kit with a million rows, as large as its surface, which is not on screen: the
/// wheel scrolls it from its first row to its last, at one speed, in as many frames as a
/// scenario runs. Every frame shows rows that the frame before did not, so every row slot is
/// bound to another row and every cell sets another text.
#[derive(Debug)]
pub struct MillionRows {
    runtime: Runtime<Surfaces>,
    surfaces: Surfaces,
    surface: SurfaceId,
    text_shaper: CoreTextShaper,
    target: OffscreenTarget,
}

impl MillionRows {
    /// Mounts the table under the light theme of the kit, on a surface whose icons the
    /// renderer draws, and runs its first frame: a table the wheel scrolls is on screen already.
    pub fn new(renderer: &Renderer) -> Self {
        let styles = Styles::of_kit();
        let mut runtime = Runtime::new();
        let mut surfaces = Surfaces::new();
        surfaces.register_kit_icons(|svg| {
            renderer
                .add_icon(svg)
                .expect("the icons of the kit are SVG the renderer reads")
        });
        let surface = surfaces.open(&mut runtime, styles.style_sheet, styles.light);
        let root = surfaces.surface(surface).tree().root();
        surfaces.mount(&mut runtime, surface, root, table);
        let mut rows = Self {
            runtime,
            surfaces,
            surface,
            text_shaper: CoreTextShaper::new(),
            target: OffscreenTarget::new(renderer, VIEWPORT, DEVICE_SCALE),
        };
        rows.run_frame(0, 0.0);
        rows
    }

    /// Scrolls through the rows in the frames of the budget, the warm-up among them, and
    /// measures the frames after the warm-up.
    pub fn measure(&mut self, budget: &FrameBudget) -> FrameMeasurements {
        let frames = budget.warm_up_frames + budget.measured_frames;
        let scroll_per_frame = ROWS as f32 * ROW_HEIGHT / frames as f32;
        for frame in 1..=budget.warm_up_frames {
            self.run_frame(frame, scroll_per_frame);
        }
        let mut samples = Vec::with_capacity(budget.measured_frames);
        for frame in budget.warm_up_frames + 1..=frames {
            samples.push(self.run_frame(frame, scroll_per_frame));
        }
        FrameMeasurements::new(samples)
    }

    /// How far below the bottom of the viewport the slot that shows the last row ends on the
    /// surface, after the frames run so far; none while no slot shows the last row.
    pub fn last_row_below_viewport(&self) -> Option<f32> {
        let tree = self.surfaces.surface(self.surface).tree();
        let table = tree.children(tree.root())[0];
        let viewport = tree.children(table)[0];
        let content = tree.children(viewport)[0];
        let rows = tree.children(content)[0];
        let last_name = format!("Document {ROWS}");
        tree.children(rows)
            .iter()
            .find(|slot| {
                let row = tree.children(**slot)[0];
                tree.text(tree.children(row)[0]) == last_name
            })
            .map(|slot| tree.surface_box(*slot).bottom() - tree.surface_box(viewport).bottom())
    }

    /// Runs one frame: the wheel scrolls the table down by `scroll`, the rows that came into
    /// view are bound to their slots, and the stages of the pipeline run, timed one after
    /// another. The GPU draws the frame after the sample is taken.
    fn run_frame(&mut self, frame: usize, scroll: f32) -> FrameSample {
        let allocations_before = CountingAllocator::allocations_so_far();
        let started = Instant::now();
        let surface = self.surfaces.surface_mut(self.surface);
        surface.scroll_input(&ScrollInput {
            position: Point {
                x: VIEWPORT.width / 2.0,
                y: VIEWPORT.height / 2.0,
            },
            delta: Point { x: 0.0, y: scroll },
            phase: ScrollPhase::Discrete,
        });
        surface.follow_scrolls(&mut self.runtime);
        self.runtime
            .flush(&mut self.surfaces)
            .expect("the effects of a table wake no cycle");
        let tree = self.surfaces.surface_mut(self.surface).tree_mut();
        tree.resolve_styles(Duration::from_secs_f64(frame as f64 / FRAMES_PER_SECOND));
        let events_done = Instant::now();
        tree.compute_layout(VIEWPORT, DEVICE_SCALE, &mut self.text_shaper);
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

fn pixels(length: f32) -> LengthPercentageAuto {
    LengthPercentageAuto::LengthPercentage(LengthPercentage {
        pixels: Pixels(length),
        fraction: None,
        tokens: Cow::Borrowed(&[]),
    })
}

/// The table of files: four columns of text, a column of numbers among them.
fn table(mount: &mut Mount<'_>) -> ElementId {
    let column = |key, title: &str, width, alignment| TableColumn {
        key,
        title: title.to_owned(),
        width,
        alignment,
    };
    let columns = mount.runtime().signal(vec![
        column(FileColumn::Name, "Name", 300.0, CellAlignment::Start),
        column(FileColumn::Size, "Size", 110.0, CellAlignment::End),
        column(FileColumn::Modified, "Modified", 180.0, CellAlignment::Start),
        column(FileColumn::Kind, "Kind", 150.0, CellAlignment::Start),
    ]);
    let selected = mount.runtime().signal(vec![3]);
    let sort = mount.runtime().signal(None);
    let view = Table::new(
        ROWS,
        columns,
        |_, row, column, text| {
            match column {
                FileColumn::Name => write!(text, "Document {}", row + 1),
                FileColumn::Size => write!(text, "{}.{} MB", row % 900 + 1, row % 10),
                FileColumn::Modified => write!(text, "Nov {}, {:02}:{:02}", row % 30 + 1, row % 24, row % 60),
                FileColumn::Kind => write!(text, "{}", KINDS[row % KINDS.len()]),
            }
            .expect(WRITTEN);
        },
        selected,
        sort,
    )
    .label("Files")
    .mount(mount);
    mount
        .build(view)
        .inline(Declaration::Width(pixels(VIEWPORT.width)))
        .inline(Declaration::Height(pixels(VIEWPORT.height)));
    view
}
