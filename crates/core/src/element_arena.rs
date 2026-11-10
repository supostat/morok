use std::ops::{Index, IndexMut};

use crate::dirty::Dirty;
use crate::element::Element;
use crate::element_id::ElementId;

#[derive(Debug)]
struct Slot {
    generation: u32,
    element: Option<Element>,
}

/// The storage of every element of a tree; a slot freed by a removal is reused under a new generation.
#[derive(Debug, Default)]
pub(crate) struct ElementArena {
    slots: Vec<Slot>,
    free_slots: Vec<u32>,
    /// Whether a cached layout was dropped since the layout stage last ran.
    layout_is_stale: bool,
    /// Whether an element waits for the lines of its text to be laid out.
    text_is_stale: bool,
}

impl ElementArena {
    pub(crate) fn insert(&mut self, element: Element) -> ElementId {
        if let Some(index) = self.free_slots.pop() {
            let slot = &mut self.slots[index as usize];
            slot.element = Some(element);
            return ElementId::new(index, slot.generation);
        }
        let index = u32::try_from(self.slots.len()).expect("a tree holds fewer than 2^32 elements");
        self.slots.push(Slot {
            generation: 0,
            element: Some(element),
        });
        ElementId::new(index, 0)
    }

    pub(crate) fn remove(&mut self, id: ElementId) -> Element {
        assert!(self.contains(id), "{id:?} names an element that was removed");
        let slot = &mut self.slots[id.index() as usize];
        slot.generation = slot.generation.wrapping_add(1);
        self.free_slots.push(id.index());
        slot.element
            .take()
            .expect("a slot of the current generation holds an element")
    }

    pub(crate) fn contains(&self, id: ElementId) -> bool {
        self.slots
            .get(id.index() as usize)
            .is_some_and(|slot| slot.generation == id.generation() && slot.element.is_some())
    }

    /// The ID of the element that occupies a slot now.
    pub(crate) fn id_at(&self, index: u32) -> ElementId {
        ElementId::new(index, self.slots[index as usize].generation)
    }

    /// Marks the element with `own` and every ancestor with `below`, so that a walk from the root
    /// finds the element by following `below`.
    pub(crate) fn mark(&mut self, id: ElementId, own: Dirty, below: Dirty) {
        self[id].dirty.insert(own);
        let mut ancestor = self[id].parent;
        while let Some(current) = ancestor {
            let element = &mut self[current];
            if element.dirty.contains(below) {
                break;
            }
            element.dirty.insert(below);
            ancestor = element.parent;
        }
    }

    /// Whether `ancestor` is `of` or one of the elements around it.
    pub(crate) fn is_inclusive_ancestor(&self, ancestor: ElementId, of: ElementId) -> bool {
        let mut current = Some(of);
        while let Some(id) = current {
            if id == ancestor {
                return true;
            }
            current = self[id].parent;
        }
        false
    }

    /// The innermost element that is, or is around, both elements; none when either is none
    /// or the two belong to different trees.
    pub(crate) fn common_ancestor(&self, first: Option<ElementId>, second: Option<ElementId>) -> Option<ElementId> {
        let (mut first, mut second) = (first?, second?);
        let (mut first_depth, mut second_depth) = (self.depth(first), self.depth(second));
        while first_depth > second_depth {
            first = self[first].parent?;
            first_depth -= 1;
        }
        while second_depth > first_depth {
            second = self[second].parent?;
            second_depth -= 1;
        }
        while first != second {
            first = self[first].parent?;
            second = self[second].parent?;
        }
        Some(first)
    }

    fn depth(&self, id: ElementId) -> usize {
        let mut depth = 0;
        let mut ancestor = self[id].parent;
        while let Some(current) = ancestor {
            depth += 1;
            ancestor = self[current].parent;
        }
        depth
    }

    pub(crate) fn slot_count(&self) -> u32 {
        self.slots.len() as u32
    }

    pub(crate) fn text_is_stale(&self) -> bool {
        self.text_is_stale
    }

    /// Records that every text marked for it has its lines laid out.
    pub(crate) fn text_is_laid_out(&mut self) {
        self.text_is_stale = false;
    }

    /// Marks the lines of the text of the element as laid out for a text, a style or a width
    /// it no longer has.
    pub(crate) fn invalidate_text(&mut self, id: ElementId) {
        self[id].dirty.insert(Dirty::TEXT);
        self.text_is_stale = true;
    }

    pub(crate) fn layout_is_stale(&self) -> bool {
        self.layout_is_stale
    }

    /// Records that the layout stage has laid out everything whose cached layout was dropped.
    pub(crate) fn layout_is_computed(&mut self) {
        self.layout_is_stale = false;
    }

    /// Drops the layout cached for the element and for every ancestor: the size of a box may
    /// change the size of every box around it.
    pub(crate) fn invalidate_layout(&mut self, id: ElementId) {
        self.layout_is_stale = true;
        let mut invalidated = Some(id);
        while let Some(current) = invalidated {
            let element = &mut self[current];
            element.layout_cache.clear();
            invalidated = element.parent;
        }
    }
}

impl Index<ElementId> for ElementArena {
    type Output = Element;

    fn index(&self, id: ElementId) -> &Element {
        match self.slots.get(id.index() as usize) {
            Some(Slot {
                generation,
                element: Some(element),
            }) if *generation == id.generation() => element,
            _ => panic!("{id:?} names an element that was removed"),
        }
    }
}

impl IndexMut<ElementId> for ElementArena {
    fn index_mut(&mut self, id: ElementId) -> &mut Element {
        match self.slots.get_mut(id.index() as usize) {
            Some(Slot {
                generation,
                element: Some(element),
            }) if *generation == id.generation() => element,
            _ => panic!("{id:?} names an element that was removed"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::ElementArena;
    use crate::element::Element;
    use crate::role::Role;

    #[test]
    fn a_removed_element_retires_its_id_and_frees_its_slot_for_a_new_generation() {
        let mut arena = ElementArena::default();
        let first = arena.insert(Element::new(Role::Element));

        arena.remove(first);
        let second = arena.insert(Element::new(Role::Text));

        assert!(!arena.contains(first));
        assert!(arena.contains(second));
        assert_eq!(first.index(), second.index());
        assert_ne!(first, second);
        assert_eq!(arena[second].role, Role::Text);
    }

    #[test]
    #[should_panic(expected = "names an element that was removed")]
    fn reading_through_a_retired_id_panics() {
        let mut arena = ElementArena::default();
        let removed = arena.insert(Element::new(Role::Element));
        arena.remove(removed);
        arena.insert(Element::new(Role::Element));

        let _ = &arena[removed];
    }
}
