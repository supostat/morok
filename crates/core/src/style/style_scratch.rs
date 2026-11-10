use crate::motion::motion_scratch::MotionScratch;
use crate::style::style_lists::StyleLists;

/// Space the style stage works in, kept from frame to frame so that it allocates nothing once
/// the lists have reached their size.
#[derive(Debug, Default)]
pub(crate) struct StyleScratch {
    /// The rules that matched the element being styled, in cascade order.
    pub(crate) matched_rules: Vec<u32>,
    /// The lists of the style the cascade gives the element.
    pub(crate) base_lists: StyleLists,
    /// The lists of the style the element shows.
    pub(crate) shown_lists: StyleLists,
    pub(crate) motion: MotionScratch,
}
