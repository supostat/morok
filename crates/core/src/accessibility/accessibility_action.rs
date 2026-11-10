/// What an assistive technology asks of an element. Taking focus is not among these: the tree
/// moves focus itself.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum AccessibilityAction<'a> {
    /// The default action of the element: what a click does.
    Click,
    Increment,
    Decrement,
    SetText(&'a str),
    SetNumber(f64),
}
