use std::fmt::{self, Debug, Formatter};
use std::time::Duration;

use crate::geometry::size::Size;
use crate::media::image_statuses::ImageStatuses;
use crate::text::text_shaper::TextShaper;

/// What a frame of a tree runs against: the surface it is laid out in, the time it is shown at,
/// and the ports of the host that measure its text and tell how far its pictures have decoded.
pub struct TreeFrame<'a> {
    pub viewport: Size,
    /// Device pixels to a CSS pixel on the surface.
    pub scale: f32,
    /// The time the frame is shown at, on the clock of the host.
    pub now: Duration,
    pub text_shaper: &'a mut dyn TextShaper,
    pub images: &'a dyn ImageStatuses,
}

impl Debug for TreeFrame<'_> {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("TreeFrame")
            .field("viewport", &self.viewport)
            .field("scale", &self.scale)
            .field("now", &self.now)
            .finish_non_exhaustive()
    }
}
