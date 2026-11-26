/// How an element is placed: in the flow, out of it, or held in view while an ancestor scrolls.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Position {
    Relative,
    Absolute,
    /// In flow, and kept inside the scrollport of the nearest scrolling ancestor by its insets
    /// as that ancestor scrolls.
    Sticky,
}
