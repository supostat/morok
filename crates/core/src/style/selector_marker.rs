use morok_style::States;

use crate::dirty::Dirty;
use crate::element_arena::ElementArena;
use crate::element_id::ElementId;
use crate::frame_requests::FrameRequests;
use crate::style::selector_change::SelectorChange;
use crate::style::style_resolver::StyleResolver;

/// Changes what selectors match of an element and marks for the style stage what the change
/// restyles.
#[derive(Debug)]
pub(crate) struct SelectorMarker<'a> {
    pub(crate) elements: &'a mut ElementArena,
    pub(crate) resolver: StyleResolver<'a>,
    pub(crate) frame_requests: &'a mut FrameRequests,
}

impl SelectorMarker<'_> {
    /// Turns the given states of the element on or off; its other states stay as they are.
    pub(crate) fn set_state(&mut self, element: ElementId, state: States, is_on: bool) {
        let before = self.elements[element].states;
        let after = if is_on {
            before.union(state)
        } else {
            before.difference(state)
        };
        if after == before {
            return;
        }
        self.elements[element].states = after;
        self.mark(element, SelectorChange::States { before });
    }

    /// Turns the given states on or off for `innermost` and each of its ancestors below `until`.
    pub(crate) fn set_state_up_to(
        &mut self,
        innermost: Option<ElementId>,
        until: Option<ElementId>,
        state: States,
        is_on: bool,
    ) {
        let mut current = innermost;
        while let Some(element) = current
            && current != until
        {
            self.set_state(element, state, is_on);
            current = self.elements[element].parent;
        }
    }

    /// Marks for the style stage the element and those of its children that a child rule
    /// styles differently after the change.
    pub(crate) fn mark(&mut self, element: ElementId, change: SelectorChange) {
        if !self.elements[element].is_attached {
            return;
        }
        self.frame_requests.request();
        self.elements.mark(element, Dirty::STYLE, Dirty::STYLE_BELOW);
        for position in 0..self.elements[element].children.len() {
            let parent = &self.elements[element];
            let child = parent.children[position];
            let (before, after) = change.targets(parent);
            let child_target = self.elements[child].selector_target();
            if self
                .resolver
                .child_rules_tell_parents_apart(child_target, before, after)
            {
                self.elements.mark(child, Dirty::STYLE, Dirty::STYLE_BELOW);
            }
        }
    }
}
