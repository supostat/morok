use morok_style::States;

use crate::element_arena::ElementArena;
use crate::element_id::ElementId;
use crate::input::focus_direction::FocusDirection;
use crate::input::focusability::Focusability;

/// The order in which the Tab key walks the tab stops: the order of the tree, a parent before
/// its children. An element that is not rendered is passed by with everything inside it.
#[derive(Clone, Copy, Debug)]
pub(crate) struct TabOrder<'a> {
    pub(crate) elements: &'a ElementArena,
    /// The subtree the walk stays in.
    pub(crate) scope: ElementId,
}

impl TabOrder<'_> {
    /// The tab stop after, or before, `from`; after the last one comes the first again. From
    /// nowhere the walk begins at the first stop, or at the last one going backward. None when
    /// there is no other stop to go to.
    pub(crate) fn stop_from(self, from: Option<ElementId>, direction: FocusDirection) -> Option<ElementId> {
        let after_from = from.and_then(|from| self.stops_after(from, direction, None));
        after_from.or_else(|| self.stops_after_the_edge(direction, from))
    }

    /// The first stop on the way from `from` to the end of the scope, short of `end`.
    fn stops_after(self, from: ElementId, direction: FocusDirection, end: Option<ElementId>) -> Option<ElementId> {
        let mut current = self.neighbor(from, direction);
        while let Some(element) = current
            && current != end
        {
            if self.is_tab_stop(element) {
                return Some(element);
            }
            current = self.neighbor(element, direction);
        }
        None
    }

    /// The first stop on the way from the edge of the scope the walk begins at, short of `end`.
    fn stops_after_the_edge(self, direction: FocusDirection, end: Option<ElementId>) -> Option<ElementId> {
        let edge = match direction {
            FocusDirection::Forward => self.scope,
            FocusDirection::Backward => self.last_within(self.scope),
        };
        if Some(edge) == end {
            return None;
        }
        if self.is_tab_stop(edge) {
            return Some(edge);
        }
        self.stops_after(edge, direction, end)
    }

    fn is_tab_stop(self, id: ElementId) -> bool {
        let element = &self.elements[id];
        element.focusability == Focusability::TabStop
            && element.is_rendered
            && !element.states.contains(States::DISABLED)
    }

    /// The element next to `id` in tree order, inside the scope.
    fn neighbor(self, id: ElementId, direction: FocusDirection) -> Option<ElementId> {
        match direction {
            FocusDirection::Forward => self.following(id),
            FocusDirection::Backward => self.preceding(id),
        }
    }

    fn following(self, id: ElementId) -> Option<ElementId> {
        let element = &self.elements[id];
        if element.is_rendered
            && let Some(first_child) = element.children.first()
        {
            return Some(*first_child);
        }
        let mut current = id;
        while current != self.scope {
            let parent = self.elements[current].parent?;
            let siblings = &self.elements[parent].children;
            if let Some(next_sibling) = siblings.get(position_among(siblings, current) + 1) {
                return Some(*next_sibling);
            }
            current = parent;
        }
        None
    }

    fn preceding(self, id: ElementId) -> Option<ElementId> {
        if id == self.scope {
            return None;
        }
        let parent = self.elements[id].parent?;
        let siblings = &self.elements[parent].children;
        match position_among(siblings, id) {
            0 => Some(parent),
            position => Some(self.last_within(siblings[position - 1])),
        }
    }

    /// The element that tree order puts last among `id` and what is rendered inside it.
    fn last_within(self, id: ElementId) -> ElementId {
        let mut last = id;
        while self.elements[last].is_rendered
            && let Some(last_child) = self.elements[last].children.last()
        {
            last = *last_child;
        }
        last
    }
}

fn position_among(siblings: &[ElementId], id: ElementId) -> usize {
    siblings
        .iter()
        .position(|sibling| *sibling == id)
        .expect("an element is among the children of its parent")
}
