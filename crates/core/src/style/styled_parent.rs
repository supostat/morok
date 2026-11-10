use morok_style::Color;

use crate::style::selector_target::SelectorTarget;
use crate::style::text_style::TextStyle;

/// What the style of a child takes from its parent: the target of child rules, the inherited
/// values and whether it is rendered.
#[derive(Clone, Copy, Debug)]
pub(crate) struct StyledParent<'a> {
    pub(crate) target: SelectorTarget<'a>,
    pub(crate) color: Color,
    pub(crate) text: TextStyle,
    /// Whether the parent is rendered: neither it nor an ancestor has `display: none`.
    pub(crate) is_rendered: bool,
}
