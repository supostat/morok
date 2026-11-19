/// Where a key moves the highlight of a dropdown list.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum DropdownStep {
    Next,
    Previous,
    First,
    Last,
}
