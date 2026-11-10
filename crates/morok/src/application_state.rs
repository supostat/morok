use morok_components::view::Surfaces;
use morok_platform::CoreTextShaper;
use morok_reactive::Runtime;

use crate::styles::Styles;

/// What the surfaces of an application share: the signal graph, the trees its effects act on,
/// and the shaper that measures their text. The callbacks of the system reach it through the
/// content of each window.
#[derive(Debug)]
pub(crate) struct ApplicationState {
    pub(crate) runtime: Runtime<Surfaces>,
    pub(crate) surfaces: Surfaces,
    pub(crate) text_shaper: CoreTextShaper,
    pub(crate) styles: Styles,
}

impl ApplicationState {
    pub(crate) fn new(styles: Styles) -> Self {
        Self {
            runtime: Runtime::new(),
            surfaces: Surfaces::new(),
            text_shaper: CoreTextShaper::new(),
            styles,
        }
    }

    /// Runs the effects that wait.
    ///
    /// # Panics
    ///
    /// When an effect keeps waking itself: the application wrote a signal in an effect that
    /// reads it.
    pub(crate) fn flush(&mut self) {
        if let Err(cycle) = self.runtime.flush(&mut self.surfaces) {
            panic!("{cycle}");
        }
    }
}
