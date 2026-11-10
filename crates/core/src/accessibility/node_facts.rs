use crate::geometry::rect::Rect;

/// Everything of a node that can change without its element being told to change it, with
/// the count of the changes it was told of. Two equal sets of facts make the same node.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) struct NodeFacts {
    /// How many times the role, the name, the value, the actions or the text of the element
    /// were changed.
    pub(crate) revision: u64,
    /// The box of the element as the surface shows it, in CSS pixels.
    pub(crate) bounds: Rect,
    pub(crate) is_disabled: bool,
    pub(crate) is_checked: bool,
    pub(crate) takes_focus: bool,
    /// For the root: device pixels to a CSS pixel on the surface.
    pub(crate) scale: Option<f32>,
}
