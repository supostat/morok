use crate::length_percentage::LengthPercentage;

/// One function of a `transform` list, applied in the order written. Percentages of a
/// translation are of the border box of the element.
#[derive(Clone, Debug, PartialEq)]
pub enum TransformFunction {
    Translate(LengthPercentage, LengthPercentage),
    TranslateX(LengthPercentage),
    TranslateY(LengthPercentage),
    Scale(f32, f32),
    ScaleX(f32),
    ScaleY(f32),
    /// Degrees clockwise.
    Rotate(f32),
}
