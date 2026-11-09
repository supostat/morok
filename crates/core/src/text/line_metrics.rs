use crate::style::text_style::TextStyle;
use crate::text::font_metrics::FontMetrics;

/// The line box of CSS for a font: its height and where the baseline lies in it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct LineMetrics {
    pub(crate) line_height: f32,
    /// From the top of the line box down to the baseline.
    pub(crate) baseline: f32,
}

impl LineMetrics {
    /// The space a line height leaves over the content area of the font, its leading, is
    /// split in halves above and below the content area.
    pub(crate) fn of(style: &TextStyle, font: &FontMetrics) -> Self {
        let content_height = font.ascent + font.descent;
        let line_height = style
            .line_height
            .map_or(content_height + font.line_gap, |multiple| multiple * style.font_size);
        Self {
            line_height,
            baseline: (line_height - content_height) / 2.0 + font.ascent,
        }
    }
}
