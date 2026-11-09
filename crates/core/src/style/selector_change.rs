use morok_style::States;

use crate::element::Element;
use crate::style::selector_target::SelectorTarget;

/// A change to what selectors match of one element, described so that both sides of it can be
/// read from the element itself.
#[derive(Clone, Copy, Debug)]
pub(crate) enum SelectorChange {
    /// The last class of the element was just added.
    ClassAdded,
    /// The last class of the element is about to be removed.
    ClassLeaving,
    States {
        before: States,
    },
}

impl SelectorChange {
    /// What selectors matched before the change and what they match after it.
    pub(crate) fn targets(self, element: &Element) -> (SelectorTarget<'_>, SelectorTarget<'_>) {
        let current = element.selector_target();
        match self {
            Self::ClassAdded => (without_last_class(element), current),
            Self::ClassLeaving => (current, without_last_class(element)),
            Self::States { before } => (
                SelectorTarget {
                    classes: &element.classes,
                    states: before,
                },
                current,
            ),
        }
    }
}

fn without_last_class(element: &Element) -> SelectorTarget<'_> {
    let (_, kept_classes) = element
        .classes
        .split_last()
        .expect("the class that changes is the last class of the element");
    SelectorTarget {
        classes: kept_classes,
        states: element.states,
    }
}
