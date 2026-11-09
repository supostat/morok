use morok_style::{Class, CompoundSelector, States};

/// What a compound selector is matched against: the classes and states of one element.
#[derive(Clone, Copy, Debug)]
pub(crate) struct SelectorTarget<'a> {
    pub(crate) classes: &'a [Class],
    pub(crate) states: States,
}

impl SelectorTarget<'_> {
    pub(crate) fn matches(self, selector: &CompoundSelector) -> bool {
        self.states.contains(selector.states) && selector.classes.iter().all(|class| self.classes.contains(class))
    }
}
