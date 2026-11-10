#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PointerButton {
    /// The button that presses and drags: the left one of a mouse.
    Primary,
    /// The button that asks for a context menu: the right one of a mouse.
    Secondary,
    /// Any further button, the wheel of a mouse among them.
    Auxiliary,
}
