use morok_components::view::Surfaces;
use morok_platform::{CoreTextShaper, Renderer};
use morok_reactive::Runtime;

use crate::styles::Styles;

/// What the surfaces of an application share: the signal graph, the trees its effects act on,
/// the shaper that measures their text and the renderer that decodes their pictures. The
/// callbacks of the system reach it through the content of each window.
#[derive(Debug)]
pub(crate) struct ApplicationState {
    pub(crate) runtime: Runtime<Surfaces>,
    pub(crate) surfaces: Surfaces,
    pub(crate) text_shaper: CoreTextShaper,
    pub(crate) renderer: Renderer,
    pub(crate) styles: Styles,
}

impl ApplicationState {
    pub(crate) fn new(styles: Styles, renderer: Renderer) -> Self {
        Self {
            runtime: Runtime::new(),
            surfaces: Surfaces::new(),
            text_shaper: CoreTextShaper::new(),
            renderer,
            styles,
        }
    }
}
