use std::borrow::Cow;

use crate::class::Class;
use crate::states::States;

/// What one element must have to match: every class and every state listed.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct CompoundSelector {
    pub classes: Cow<'static, [Class]>,
    pub states: States,
}
