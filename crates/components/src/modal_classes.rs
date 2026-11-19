use morok_style::Class;

/// The classes of the parts of a modal panel: a dialog and a sheet are built alike and
/// styled apart.
#[derive(Clone, Copy, Debug)]
pub(crate) struct ModalClasses {
    /// The scrim that covers the surface and holds the panel.
    pub(crate) layer: Class,
    pub(crate) panel: Class,
    pub(crate) title: Class,
    pub(crate) message: Class,
    pub(crate) content: Class,
    pub(crate) actions: Class,
}
