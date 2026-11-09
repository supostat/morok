use morok_style::Color;

use crate::style::selector_target::SelectorTarget;

/// What the style of a child takes from its parent: the target of child rules and the inherited values.
#[derive(Clone, Copy, Debug)]
pub(crate) struct StyledParent<'a> {
    pub(crate) target: SelectorTarget<'a>,
    pub(crate) color: Color,
}
