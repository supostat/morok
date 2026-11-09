use morok_platform::{CoreTextShaper, OffscreenTarget, Renderer};
use morok_style::ColorScheme;

use crate::case::Case;
use crate::case_rendering::CaseRendering;
use crate::compiled_suite::CompiledSuite;
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

    /// Draws the case under both color schemes.
    pub fn render_case(&mut self, suite: &CompiledSuite<'_>, case: &Case) -> CaseRendering {
        CaseRendering {
            light: self.render(suite, case, ColorScheme::Light),
            dark: self.render(suite, case, ColorScheme::Dark),
        }
    }

    fn render(&mut self, suite: &CompiledSuite<'_>, case: &Case, color_scheme: ColorScheme) -> Rendering {
        let mut scene = suite.scene(case, color_scheme);
        scene.frame(case.viewport, &mut self.text_shaper);
        let tree = scene.tree();
        Rendering {
            image: OffscreenTarget::new(&self.renderer, case.viewport, DEVICE_SCALE).render(tree.display_list()),
            boxes: scene.element_boxes(scene.elements().into_iter().map(|element| tree.surface_box(element))),
        }
    }
}
