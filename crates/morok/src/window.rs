use morok_components::view::SurfaceId;
use morok_core::{ElementTree, Size};
use morok_platform::FrameStatistics;

use crate::window_content::WindowContent;

/// A window of an application: one surface with a tree of its own, on screen for as long as
/// this value lives. A window is not dropped while the application handles an event or runs
/// [`Application::update`](crate::Application::update).
#[derive(Debug)]
pub struct Window {
    pub(crate) platform: morok_platform::Window<WindowContent>,
    pub(crate) surface: SurfaceId,
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

    /// Reads or changes the tree of the window directly. A change the tree notices asks for a
    /// frame by itself.
    pub fn with_tree<Outcome>(&self, access: impl FnOnce(&mut ElementTree) -> Outcome) -> Outcome {
        self.platform.update(|content| {
            let state = &mut *content.state.borrow_mut();
            access(state.surfaces.surface_mut(self.surface).tree_mut())
        })
    }
}
