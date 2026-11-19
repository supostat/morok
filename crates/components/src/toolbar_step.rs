/// Where a key moves focus among the controls of a toolbar.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ToolbarStep {
    Next,
    Previous,
    First,
    Last,
}
