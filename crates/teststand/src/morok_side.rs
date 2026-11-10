use std::thread;
use std::time::{Duration, Instant};

use morok_platform::{CoreTextShaper, ImageStatus, OffscreenTarget, Renderer};
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
            let size = self.decoded_size(id);
            scene.tree_mut().set_image(element, id, Some(size));
        }
        scene.frame(case.viewport, &mut self.text_shaper, Duration::ZERO);
        if let Moment::InMotion { change, after } = case.moment {
            change(&mut scene);
            scene.frame(case.viewport, &mut self.text_shaper, Duration::ZERO);
            scene.frame(case.viewport, &mut self.text_shaper, after);
        }
        let tree = scene.tree();
        Rendering {
            image: OffscreenTarget::new(&self.renderer, case.viewport, DEVICE_SCALE).render(tree.display_list()),
            boxes: scene.element_boxes(scene.elements().into_iter().map(|element| tree.surface_box(element))),
        }
    }

    /// Waits for the decoder of the image: a case is drawn with its pictures in place.
    fn decoded_size(&self, image: morok_core::ImageId) -> morok_core::Size {
        let started = Instant::now();
        loop {
            match self.renderer.image_status(image) {
                ImageStatus::Decoded(size) => return size,
                ImageStatus::Unreadable => panic!("the images of a suite are files the system reads"),
                ImageStatus::Decoding => {
                    assert!(
                        started.elapsed() < DECODING_TIMEOUT,
                        "an image of a suite decodes in time"
                    );
                    thread::sleep(DECODING_POLL);
                }
            }
        }
    }
}

const DECODING_TIMEOUT: Duration = Duration::from_secs(10);
const DECODING_POLL: Duration = Duration::from_millis(2);
