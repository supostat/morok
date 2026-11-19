use morok_core::ElementId;

/// The elements of a built field of the kit.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct FieldElements {
    /// The box of the class `field`.
    pub(crate) field: ElementId,
    /// The text the user edits, inside the box.
    pub(crate) input: ElementId,
}
