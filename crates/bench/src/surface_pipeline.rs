use std::fmt::{self, Debug, Formatter};
use std::time::Duration;

use morok::Styles;
use morok::components::view::{Mount, Surface, SurfaceFrame, SurfaceId, Surfaces};
use morok::core::{ElementId, ElementTree, Size, TreeFrame};
use morok::reactive::Runtime;
use morok_platform::{CanvasPainter, CoreTextShaper, OffscreenTarget, Renderer};

use crate::frame_sample::FrameSample;
use crate::stage_clock::StageClock;

/// Frames are measured as a Retina display shows them.
const DEVICE_SCALE: f32 = 2.0;
/// The frames follow one another at the rate of a `ProMotion` display.
const FRAMES_PER_SECOND: f64 = 120.0;
/// The frames a surface runs before it is used.
const SETTLING_FRAMES: usize = 2;

/// A surface of components that is not on screen, with the frame of the kit over it: a frame
/// applies the input of a scenario and runs the frame of the surface, its pictures sized by
/// the renderer, timed and with its heap allocations counted by budget stage.
pub(crate) struct SurfacePipeline {
    runtime: Runtime<Surfaces>,
    surfaces: Surfaces,
    surface: SurfaceId,
    viewport: Size,
    text_shaper: CoreTextShaper,
    renderer: Renderer,
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
            renderer: renderer.clone(),
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

    /// Runs frame number `frame`: `input` reaches the surface, then the frame of the surface
    /// runs, then the encoding. The GPU draws the frame after the sample is taken.
    ///
    /// # Panics
    ///
    /// When the effects of the frame wake one another in a cycle, or close the surface.
    pub(crate) fn run_frame(&mut self, frame: usize, input: impl FnOnce(&mut Surface)) -> FrameSample {
        let mut clock = StageClock::start();
        input(self.surfaces.surface_mut(self.surface));
        let tree_frame = TreeFrame {
            viewport: self.viewport,
            scale: DEVICE_SCALE,
            now: Duration::from_secs_f64(frame as f64 / FRAMES_PER_SECOND),
            text_shaper: &mut self.text_shaper,
            images: &self.renderer,
        };
        match self
            .surfaces
            .run_frame(&mut self.runtime, self.surface, tree_frame, &mut clock)
        {
            Ok(SurfaceFrame::Ran(_)) => {}
            Ok(SurfaceFrame::Closed) => panic!("a scenario closed the surface it measures"),
            Err(cycle) => panic!("{cycle}"),
        }
        let tree = self.surfaces.surface(self.surface).tree();
        self.target.draw(tree.display_list(), tree.theme());
        let sample = clock.encoding_ended();
        self.target.wait_until_drawn();
        sample
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
