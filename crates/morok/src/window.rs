use std::rc::Rc;

use morok_components::view::SurfaceId;
use morok_core::{ElementTree, Size};
use morok_platform::{FrameImage, FrameStatistics, OffscreenTarget};

use crate::hosted_surface::HostedSurface;
use crate::shared_canvas_painter::SharedCanvasPainter;

/// A window of an application: one surface with a tree of its own, on screen for as long as
/// this value lives. A window is not dropped while the application handles an event or runs
/// [`Application::update`](crate::Application::update).
#[derive(Debug)]
pub struct Window {
    pub(crate) platform: morok_platform::Window<HostedSurface>,
    pub(crate) surface: SurfaceId,
    /// The application whose renderer draws the window.
    pub(crate) application: Rc<morok_platform::Application>,
    pub(crate) canvas_painter: SharedCanvasPainter,
}

impl Window {
    /// The size of the surface in CSS pixels.
    pub fn viewport(&self) -> Size {
        self.platform.viewport()
    }

    /// Resizes the window so that its surface takes `content_size` CSS pixels.
    pub fn set_content_size(&self, content_size: Size) {
        self.platform.set_content_size(content_size);
    }

    pub fn frame_statistics(&self) -> FrameStatistics {
        self.platform.frame_statistics()
    }

    /// The last frame of the window drawn again off screen, as the renderer of the application
    /// drew it on screen: the display list the tree built last, at the scale of the screen the
    /// window is on, its canvases drawn by the painter of the application. The overlays opened
    /// over the window are windows of their own and are not in it.
    pub fn frame_image(&self) -> FrameImage {
        let mut target = OffscreenTarget::new(self.application.renderer(), self.viewport(), self.platform.scale());
        target.set_canvas_painter(Box::new(self.canvas_painter.clone()));
        self.platform.update(|content| {
            let state = content.state.borrow();
            let tree = state.surfaces.surface(self.surface).tree();
            target.render(tree.display_list(), tree.theme())
        })
    }

    /// Reads or changes the tree of the window directly. A change the tree notices asks for a
    /// frame by itself.
    pub fn with_tree<Outcome>(&self, access: impl FnOnce(&mut ElementTree) -> Outcome) -> Outcome {
        self.platform.update(|content| {
            let state = &mut *content.state.borrow_mut();
            access(state.surfaces.surface_mut(self.surface).tree_mut())
        })
    }
}
