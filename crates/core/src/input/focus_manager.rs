use morok_style::States;

use crate::element_id::ElementId;
use crate::input::focus_direction::FocusDirection;
use crate::input::focus_trap::FocusTrap;
use crate::input::focusability::Focusability;
use crate::input::input_modality::InputModality;
use crate::input::input_state::InputState;
use crate::input::tab_order::TabOrder;
use crate::style::selector_marker::SelectorMarker;

/// Moves keyboard focus and keeps the focus states of the elements in step with it: `focus`
/// on the focused element, `focus-visible` on it while the user works with the keyboard, and
/// `focus-within` on it and on every element around it.
#[derive(Debug)]
pub(crate) struct FocusManager<'a> {
    pub(crate) marker: SelectorMarker<'a>,
    pub(crate) input: &'a mut InputState,
    pub(crate) root: ElementId,
}

impl FocusManager<'_> {
    /// Gives focus to the element. Returns whether it has focus now: an element that is not in
    /// the tree, is not focusable, is disabled or lies outside the innermost trap takes none.
    pub(crate) fn focus(&mut self, element: ElementId) -> bool {
        if !self.takes_focus(element) {
            return false;
        }
        self.set_focused(Some(element));
        true
    }

    pub(crate) fn blur(&mut self) {
        self.set_focused(None);
    }

    pub(crate) fn set_modality(&mut self, modality: InputModality) {
        if self.input.modality == modality {
            return;
        }
        self.input.modality = modality;
        if let Some(focused) = self.input.focused {
            let shows_focus = modality == InputModality::Keyboard;
            self.marker.set_state(focused, States::FOCUS_VISIBLE, shows_focus);
        }
    }

    /// A pointer button went down over `hit`: the innermost element around the place that
    /// takes focus gets it. A press on nothing focusable leaves focus where it is.
    pub(crate) fn pointer_pressed(&mut self, hit: Option<ElementId>) {
        self.set_modality(InputModality::Pointer);
        let mut current = hit;
        while let Some(element) = current {
            if self.focus(element) {
                return;
            }
            current = self.marker.elements[element].parent;
        }
    }

    /// Moves focus to the next tab stop in the direction, inside the innermost trap. Returns
    /// whether there was a stop to move to.
    pub(crate) fn move_focus(&mut self, direction: FocusDirection) -> bool {
        let order = TabOrder {
            elements: self.marker.elements,
            scope: self.scope(),
        };
        let Some(stop) = order.stop_from(self.input.focused, direction) else {
            return false;
        };
        self.set_focused(Some(stop));
        true
    }

    /// Keeps focus inside the subtree until the trap is released. Focus that is outside the
    /// subtree now is dropped, and remembered for the release.
    pub(crate) fn trap(&mut self, root: ElementId) {
        self.input.focus_traps.push(FocusTrap {
            root,
            returns_to: self.input.focused,
        });
        if self.input.focused.is_some_and(|focused| !self.is_inside(root, focused)) {
            self.blur();
        }
    }

    /// Releases the trap of the subtree. Releasing the innermost trap gives focus back to the
    /// element that had it when the trap was set, if it still takes focus.
    ///
    /// # Panics
    ///
    /// When the subtree traps no focus.
    pub(crate) fn release_trap(&mut self, root: ElementId) {
        let position = self
            .input
            .focus_traps
            .iter()
            .rposition(|trap| trap.root == root)
            .expect("the element traps focus until the trap is released");
        let trap = self.input.focus_traps.remove(position);
        let was_innermost = position == self.input.focus_traps.len();
        if was_innermost
            && let Some(returns_to) = trap.returns_to
            && self.marker.elements.contains(returns_to)
        {
            self.focus(returns_to);
        }
    }

    /// The subtree is about to leave the tree: focus inside it is dropped, and so are the traps
    /// set inside it.
    pub(crate) fn forget_subtree(&mut self, subtree: ElementId) {
        let elements = &*self.marker.elements;
        self.input
            .focus_traps
            .retain(|trap| !elements.is_inclusive_ancestor(subtree, trap.root));
        if self
            .input
            .focused
            .is_some_and(|focused| self.is_inside(subtree, focused))
        {
            self.blur();
        }
    }

    /// Drops the focus of an element that is no longer rendered.
    pub(crate) fn drop_hidden_focus(&mut self) {
        if self
            .input
            .focused
            .is_some_and(|focused| !self.marker.elements[focused].is_rendered)
        {
            self.blur();
        }
    }

    /// The subtree focus moves in: that of the innermost trap, or the whole tree.
    fn scope(&self) -> ElementId {
        self.input.focus_traps.last().map_or(self.root, |trap| trap.root)
    }

    fn is_inside(&self, subtree: ElementId, element: ElementId) -> bool {
        self.marker.elements.is_inclusive_ancestor(subtree, element)
    }

    fn takes_focus(&self, id: ElementId) -> bool {
        let element = &self.marker.elements[id];
        element.is_attached
            && element.focusability != Focusability::NotFocusable
            && !element.states.contains(States::DISABLED)
            && self.is_inside(self.scope(), id)
    }

    fn set_focused(&mut self, focused: Option<ElementId>) {
        let before = self.input.focused;
        if before == focused {
            return;
        }
        let kept = self.marker.elements.common_ancestor(before, focused);
        if let Some(before) = before {
            let own_states = States::FOCUS.union(States::FOCUS_VISIBLE);
            self.marker.set_state(before, own_states, false);
        }
        self.marker.set_state_up_to(before, kept, States::FOCUS_WITHIN, false);
        self.marker.set_state_up_to(focused, kept, States::FOCUS_WITHIN, true);
        if let Some(focused) = focused {
            let own_states = match self.input.modality {
                InputModality::Keyboard => States::FOCUS.union(States::FOCUS_VISIBLE),
                InputModality::Pointer => States::FOCUS,
            };
            self.marker.set_state(focused, own_states, true);
        }
        self.input.focused = focused;
    }
}
