use crate::element_id::ElementId;

/// Where focus has to be for a shortcut to answer.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ShortcutScope {
    /// Anywhere on the surface, a window for one, and also when nothing has focus.
    Surface,
    /// On the element or inside it.
    Element(ElementId),
}
