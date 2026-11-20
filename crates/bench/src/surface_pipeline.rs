use std::fmt::{self, Debug, Formatter};
use std::time::{Duration, Instant};

use morok::Styles;
use morok::components::view::{Mount, Surface, SurfaceId, Surfaces};
use morok::core::{ElementId, ElementTree, Size};
use morok::reactive::Runtime;
use morok_platform::{CanvasPainter, CoreTextShaper, OffscreenTarget, Renderer};

use crate::counting_allocator::CountingAllocator;
use crate::frame_sample::FrameSample;

/// Frames are measured as a Retina display shows them.
const DEVICE_SCALE: f32 = 2.0;
/// The frames follow one another at the rate of a `ProMotion` display.
const FRAMES_PER_SECOND: f64 = 120.0;
/// The frames a surface runs before it is used.
const SETTLING_FRAMES: usize = 2;

/// A surface of components that is not on screen, with the frame pipeline of the kit over it:
/// a frame applies the input of a scenario, runs the effects it woke, and the stages of the
/// tree, timed one after another, with the heap allocations they make counted.
pub(crate) struct SurfacePipeline {
    runtime: Runtime<Surfaces>,
    surfaces: Surfaces,
    surface: SurfaceId,
    viewport: Size,
    text_shaper: CoreTextShaper,
    target: OffscreenTarget,
}

impl SurfacePipeline {
    /// Mounts `root` under the light theme of `styles`, on a surface of `viewport` whose icons
    /// and canvases the renderer draws, and runs the frames a surface on screen has run by the
    /// time it is used: the one that lays it out, and the one that binds what its components
    /// show to the room the layout gave them — the rows a virtualized collection shows.
    pub(crate) fn new(
        renderer: &Renderer,
        styles: Styles,
        viewport: Size,
        canvas_painter: Option<Box<dyn CanvasPainter>>,
        root: impl FnOnce(&mut Mount<'_>) -> ElementId,
    ) -> Self {
        let mut runtime = Runtime::new();
        let mut surfaces = Surfaces::new();
        surfaces.register_kit_icons(|svg| {
            renderer
                .add_icon(svg)
                .expect("the icons of the kit are SVG the renderer reads")
        });
        let surface = surfaces.open(&mut runtime, styles.style_sheet, styles.light);
        let tree_root = surfaces.surface(surface).tree().root();
        surfaces.mount(&mut runtime, surface, tree_root, root);
        let mut target = OffscreenTarget::new(renderer, viewport, DEVICE_SCALE);
        if let Some(painter) = canvas_painter {
            target.set_canvas_painter(painter);
        }
        let mut pipeline = Self {
            runtime,
            surfaces,
            surface,
            viewport,
            text_shaper: CoreTextShaper::new(),
            target,
        };
        for _ in 0..SETTLING_FRAMES {
            pipeline.run_frame(0, |_| {});
        }
        pipeline
    }

    pub(crate) fn tree(&self) -> &ElementTree {
        self.surfaces.surface(self.surface).tree()
    }

    /// Runs frame number `frame`: `input` reaches the surface, the scrolls it moved are
    /// followed, the effects it woke run, and the stages of the tree run, timed one after
    /// another. The GPU draws the frame after the sample is taken.
    pub(crate) fn run_frame(&mut self, frame: usize, input: impl FnOnce(&mut Surface)) -> FrameSample {
        let allocations_before = CountingAllocator::allocations_so_far();
        let started = Instant::now();
        let surface = self.surfaces.surface_mut(self.surface);
        input(surface);
        surface.follow_scrolls(&mut self.runtime);
        self.runtime
            .flush(&mut self.surfaces)
            .expect("the effects of the components wake no cycle");
        let tree = self.surfaces.surface_mut(self.surface).tree_mut();
        tree.resolve_styles(Duration::from_secs_f64(frame as f64 / FRAMES_PER_SECOND));
        let events_done = Instant::now();
        tree.compute_layout(self.viewport, DEVICE_SCALE, &mut self.text_shaper);
        let layout_done = Instant::now();
        tree.build_display_list();
        let display_list_done = Instant::now();
        self.target.draw(tree.display_list(), tree.theme());
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

impl Debug for SurfacePipeline {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("SurfacePipeline")
            .field("viewport", &self.viewport)
            .finish_non_exhaustive()
    }
}
