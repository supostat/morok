/// How the picture of an image fills the box of its element, as CSS `object-fit` has it.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum ObjectFit {
    /// Stretched to the box.
    #[default]
    Fill,
    /// Scaled to fit inside the box, keeping its proportions.
    Contain,
    /// Scaled to cover the box, keeping its proportions; what overflows is cut.
    Cover,
    /// At its own size, centered; what overflows is cut.
    None,
    /// `none` when the picture is smaller than the box, `contain` otherwise.
    ScaleDown,
}
