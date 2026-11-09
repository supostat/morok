use crate::geometry::size::Size;
use crate::layout::text_measure_request::TextMeasureRequest;

/// The port through which layout measures text. The tree knows nothing of fonts: the platform
/// implements the port with its shaper.
pub trait TextMeasurer {
    /// The size of the text once its lines are broken under the width constraint of the request.
    fn measure(&mut self, request: &TextMeasureRequest<'_>) -> Size;
}
