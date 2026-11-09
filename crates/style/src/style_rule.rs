use std::borrow::Cow;

use crate::declaration::Declaration;
use crate::selector::Selector;

/// Declarations in source order: a later declaration of a property overrides an earlier one.
#[derive(Clone, Debug, PartialEq)]
pub struct StyleRule {
    pub selector: Selector,
    pub declarations: Cow<'static, [Declaration]>,
}
