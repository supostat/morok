use morok_style::Side;

use crate::layout::track_slot::TrackSlot;

/// Names one length of the layout style of an element.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum LengthSlot {
    Width,
    Height,
    MinWidth,
    MinHeight,
    MaxWidth,
    MaxHeight,
    Margin(Side),
    Padding(Side),
    Inset(Side),
    FlexBasis,
    RowGap,
    ColumnGap,
    Track(TrackSlot),
}
