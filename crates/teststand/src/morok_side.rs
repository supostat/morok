use std::thread;
use std::time::{Duration, Instant};

use morok_core::{ImageId, ImageStatus};
use morok_platform::{CoreTextShaper, OffscreenTarget, Renderer};
use morok_style::ColorScheme;

use crate::case::Case;
use crate::case_rendering::CaseRendering;
use crate::compiled_suite::CompiledSuite;
use crate::moment::Moment;
use crate::rendering::{DEVICE_SCALE, Rendering};
use crate::stand_error::StandError;

/// The kit as the stand drives it: the tree of a case goes through the stages of a frame and
/// is drawn offscreen by the renderer of the kit.
#[derive(Debug)]
pub struct MorokSide {
    renderer: Renderer,
    text_shaper: CoreTextShaper,
}

impl MorokSide {
    pub fn new() -> Result<Self, StandError> {
        Ok(Self {
            renderer: Renderer::new().map_err(StandError::Renderer)?,
            text_shaper: CoreTextShaper::new(),
        })
    }

    /// Draws the case under both color schemes, at its moment.
    pub fn render_case(&mut self, suite: &CompiledSuite<'_>, case: &Case) -> CaseRendering {
        CaseRendering {
            light: self.render(suite, case, ColorScheme::Light),
            dark: self.render(suite, case, ColorScheme::Dark),
        }
    }

    fn render(&mut self, suite: &CompiledSuite<'_>, case: &Case, color_scheme: ColorScheme) -> Rendering {
        let mut scene = suite.scene(case, color_scheme);
        for (element, icon) in scene.icons().to_vec() {
            let id = self
                .renderer
                .add_icon(icon.svg)
                .expect("the icons of a suite are SVG the kit reads");
            scene.tree_mut().set_icon(element, id);
        }
        for (element, image) in scene.images().to_vec() {
            let id = self.renderer.add_image(image.encoded.to_vec());
            scene.tree_mut().set_image(element, id, None);
            self.wait_for_decoder(id);
        }
        let renderer = &self.renderer;
        scene.tree_mut().size_images(|image| renderer.image_status(image));
        scene.frame(case.viewport, &mut self.text_shaper, Duration::ZERO);
        if let Moment::InMotion { change, after } = case.moment {
            change(&mut scene);
            scene.frame(case.viewport, &mut self.text_shaper, Duration::ZERO);
            scene.frame(case.viewport, &mut self.text_shaper, after);
        }
        let mut target = OffscreenTarget::new(&self.renderer, case.viewport, DEVICE_SCALE);
        target.set_canvas_painter(Box::new(scene.take_canvas_painters()));
        let tree = scene.tree();
        Rendering {
            image: target.render(tree.display_list(), tree.theme()),
            boxes: scene.element_boxes(scene.elements().into_iter().map(|element| tree.surface_box(element))),
        }
    }

    /// Waits until the decoder has read the image or found it unreadable: a case is drawn
    /// with its pictures in place.
    fn wait_for_decoder(&self, image: ImageId) {
        let started = Instant::now();
        while self.renderer.image_status(image) == ImageStatus::Decoding {
            assert!(
                started.elapsed() < DECODING_TIMEOUT,
                "an image of a suite decodes in time"
            );
            thread::sleep(DECODING_POLL);
        }
    }
}

const DECODING_TIMEOUT: Duration = Duration::from_secs(10);
const DECODING_POLL: Duration = Duration::from_millis(2);
