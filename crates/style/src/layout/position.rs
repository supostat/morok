#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Position {
    Relative,
    Absolute,
    /// In flow, and kept inside the scrollport of the nearest scrolling ancestor by its insets
    /// as that ancestor scrolls.
    Sticky,
}
