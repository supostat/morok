use crate::compound_selector::CompoundSelector;

/// Matches an element by its own classes and states and, for a child rule, by those of its parent.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Selector {
    pub subject: CompoundSelector,
    pub parent: Option<CompoundSelector>,
}
