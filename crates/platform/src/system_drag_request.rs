use morok_core::{DisplayList, Payload, Rect};
use morok_style::Theme;

/// A drag the content of a surface hands to the system: the window begins it there, under the
/// pointer that drags its source.
#[derive(Debug)]
pub struct SystemDragRequest {
    /// What the drag carries.
    pub payload: Payload,
    /// What the surface drew of the source of the drag and of nothing else, as
    /// [`ElementTree::subtree_display_list`](morok_core::ElementTree::subtree_display_list)
    /// writes it: the system shows it under the pointer.
    pub picture: DisplayList,
    /// The values of the tokens the surface draws the picture with.
    pub theme: Theme,
    /// The rectangle of the surface the picture covers, in CSS pixels; none when the surface
    /// shows nothing of the source, and the drag has no picture.
    pub picture_box: Option<Rect>,
}
