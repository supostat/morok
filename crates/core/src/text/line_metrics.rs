use crate::style::text_style::TextStyle;
use crate::text::font_metrics::FontMetrics;

/// The line box of CSS for a font: its height and where the baseline lies in it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct LineMetrics {
    pub(crate) line_height: f32,
    /// From the top of the line box down to the baseline.
    pub(crate) baseline: f32,
    /// From the baseline up to the top of the content area, in whole device pixels.
    pub(crate) ascent: f32,
    /// The height of the content area of the font, in whole device pixels.
    pub(crate) content_height: f32,
}

/// Browsers keep a length in sixty-fourths of a device pixel and cut what is finer off.
const LENGTH_UNITS_PER_DEVICE_PIXEL: f32 = 64.0;

impl LineMetrics {
    /// The space a line height leaves over the content area of the font, its leading, is
    /// split in halves above and below the content area.
    ///
    /// Browsers lay lines out in device pixels, and a baseline sits a pixel off theirs unless
    /// the same quantities are whole here: the ascent, the descent and the line gap of the
    /// font are rounded to device pixels, a computed line height is cut to the browsers' unit
    /// of length, and the half of the leading above the content area is rounded down to a
    /// device pixel. `scale` is the number of device pixels to a CSS pixel.
    pub(crate) fn of(style: &TextStyle, font: &FontMetrics, scale: f32) -> Self {
        let rounded_to_device_pixels = |length: f32| (length * scale).round() / scale;
        let ascent = rounded_to_device_pixels(font.ascent);
        let content_height = ascent + rounded_to_device_pixels(font.descent);
        let line_height = style.line_height.map_or_else(
            || content_height + rounded_to_device_pixels(font.line_gap),
            |multiple| {
                let units_per_pixel = LENGTH_UNITS_PER_DEVICE_PIXEL * scale;
                (multiple * style.font_size * units_per_pixel).trunc() / units_per_pixel
            },
        );
        let leading_above = ((line_height - content_height) / 2.0 * scale).floor() / scale;
        Self {
            line_height,
            baseline: leading_above + ascent,
            ascent,
            content_height,
        }
    }
}
