use morok_style::Class;

use crate::element_id::ElementId;

/// The overlay scroll bars of a `Scroll` element: the thumbs the tree sizes and moves along
/// their bars as the content scrolls. A thumb's parent is its bar, the track it moves along.
/// While the content moves the thumbs carry the class `showing`, and they lose it in the frame
/// after it comes to rest.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ScrollBars {
    pub vertical: Option<ElementId>,
    pub horizontal: Option<ElementId>,
    pub showing: Class,
}
