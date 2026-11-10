/// Whether an element takes keyboard focus, and how it is reached.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum Focusability {
    #[default]
    NotFocusable,
    /// Takes focus when pressed or asked to; the Tab key passes it by.
    Focusable,
    /// Takes focus as a focusable element does, and the Tab key stops on it, in tree order.
    TabStop,
}
