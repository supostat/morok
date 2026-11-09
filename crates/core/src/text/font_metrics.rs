/// What a font says about its lines, in CSS pixels at the size it was asked for.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct FontMetrics {
    /// From the baseline up to the top of the content area.
    pub ascent: f32,
    /// From the baseline down to the bottom of the content area.
    pub descent: f32,
    /// The extra space the font asks for between two lines under `line-height: normal`.
    pub line_gap: f32,
    /// From the baseline down to the top edge of an underline.
    pub underline_top: f32,
    /// From the baseline to the top edge of a line through the text; negative, it lies above.
    pub line_through_top: f32,
    pub decoration_thickness: f32,
}
