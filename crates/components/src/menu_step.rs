/// The way the arrow keys move the highlight of a menu.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum MenuStep {
    Down,
    Up,
}
