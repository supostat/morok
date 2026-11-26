use crate::paint::gradient::Gradient;

/// What a box paints over its background color.
#[derive(Clone, Debug, PartialEq)]
pub enum BackgroundImage {
    None,
    Gradient(Gradient),
}
