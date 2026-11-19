use morok_style::States;

use crate::element_id::ElementId;
use crate::input::input_state::InputState;
use crate::style::selector_marker::SelectorMarker;

/// Keeps the hover and active states of the elements in step with the pointer.
#[derive(Debug)]
pub(crate) struct PointerTargets<'a> {
    pub(crate) marker: SelectorMarker<'a>,
    pub(crate) input: &'a mut InputState,
}

impl PointerTargets<'_> {
    /// Hovers the element with its ancestors and nothing else.
    pub(crate) fn set_hovered(&mut self, hovered: Option<ElementId>) {
        let before = self.input.hovered;
        let kept = self.marker.elements.common_ancestor(before, hovered);
        self.marker.set_state_up_to(before, kept, States::HOVER, false);
        self.marker.set_state_up_to(hovered, kept, States::HOVER, true);
        self.input.hovered = hovered;
    }

    /// Makes the element with its ancestors active and nothing else.
    pub(crate) fn set_pressed(&mut self, pressed: Option<ElementId>) {
        let before = self.input.pressed;
        let kept = self.marker.elements.common_ancestor(before, pressed);
        self.marker.set_state_up_to(before, kept, States::ACTIVE, false);
        self.marker.set_state_up_to(pressed, kept, States::ACTIVE, true);
        self.input.pressed = pressed;
    }

    /// The subtree is about to leave the tree: the pointer lets go of what it holds inside it.
    /// Hover falls back on the element around the subtree until the next frame works out what
    /// the pointer is over now. A drag under way loses the target and the source it has inside
    /// the subtree, and the next frame works out the target it is over; a drag that waits for
    /// the host to hand it to the system is given up with its source.
    pub(crate) fn forget_subtree(&mut self, subtree: ElementId) {
        let elements = &*self.marker.elements;
        let is_inside =
            |held: Option<ElementId>| held.is_some_and(|held| elements.is_inclusive_ancestor(subtree, held));
        let (holds_pressed, holds_hovered) = (is_inside(self.input.pressed), is_inside(self.input.hovered));
        let holds_drag_candidate = is_inside(self.input.drag_candidate.map(|candidate| candidate.source));
        let holds_drag_source = is_inside(self.input.drag.as_ref().and_then(|drag| drag.source));
        let holds_system_drag = is_inside(self.input.system_drag.as_ref().map(|drag| drag.source));
        let around = elements[subtree].parent;
        if holds_pressed {
            self.set_pressed(None);
        }
        if holds_drag_candidate {
            self.input.drag_candidate = None;
        }
        if holds_system_drag {
            self.input.system_drag = None;
        }
        if holds_drag_source && let Some(drag) = &mut self.input.drag {
            drag.source = None;
        }
        if holds_hovered {
            let hovered_meanwhile = if self.input.drag.is_some() { None } else { around };
            self.set_hovered(hovered_meanwhile);
            self.input.hover_is_stale = true;
            self.marker.frame_requests.request();
        }
    }
}
