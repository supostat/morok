/// How much width a text may take when it is measured.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum WidthConstraint {
    /// The narrowest the text can be: a line break at every opportunity.
    MinContent,
    /// The width of the text on one line.
    MaxContent,
    /// Lines wrap at this width, in CSS pixels.
    Definite(f32),
}
