use crate::text::font_metrics::FontMetrics;
use crate::text::font_request::FontRequest;
use crate::text::shaped_text::ShapedText;
use crate::text::text_extent::TextExtent;
use crate::text::text_request::TextRequest;

/// The port through which the tree measures and shapes text. The tree knows nothing of fonts:
/// the platform implements the port with its shaper, and the tree lays the lines out as CSS
/// has it.
///
/// Both questions break lines the same way: a line exactly as wide as the constraint fits,
/// and the width of a line leaves out the white space the line ends with.
pub trait TextShaper {
    /// The metrics of the first font of the request that the system has.
    fn font_metrics(&mut self, font: &FontRequest<'_>) -> FontMetrics;

    /// How wide the text is and how many lines it takes. A text without characters takes none.
    fn measure(&mut self, request: &TextRequest<'_>) -> TextExtent;

    /// Replaces what `shaped` holds with the lines of the text. A line wider than
    /// `ellipsis_width` is cut to fit it and ends with an ellipsis.
    fn shape(&mut self, request: &TextRequest<'_>, ellipsis_width: Option<f32>, shaped: &mut ShapedText);
}
