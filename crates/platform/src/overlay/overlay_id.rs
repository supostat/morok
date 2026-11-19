/// Names an overlay to the window that shows it and to the content that asked for it: the
/// content picks the number and keeps it for as long as it wants the overlay shown.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct OverlayId(pub u64);
