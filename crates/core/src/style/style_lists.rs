use morok_style::ShadowLayer;

use crate::motion::transform_step::TransformStep;

/// The values of a style that are lists, resolved. They live beside the computed style, which
/// holds values of a fixed size only, so that a style is copied without copying a list.
#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct StyleLists {
    /// The `transform` as steps applied last to first; empty for `none`.
    pub(crate) transform: Vec<TransformStep>,
    /// The layers of `box-shadow`, the first painted on top; empty for `none`.
    pub(crate) box_shadow: Vec<ShadowLayer>,
}

impl StyleLists {
    /// Takes the values of `other`, keeping the lists already held where they have room.
    pub(crate) fn copy_from(&mut self, other: &Self) {
        self.transform.clone_from(&other.transform);
        self.box_shadow.clone_from(&other.box_shadow);
    }
}
