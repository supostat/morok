use crate::dirty::Dirty;
use crate::element_arena::ElementArena;
use crate::element_id::ElementId;
use crate::paint::damage::Damage;
use crate::paint::painted_element::PaintedElement;

/// Takes an element and its descendants out of what the surface shows: the area they drew is
/// damaged, and they rebuild their display items if they are ever painted again.
///
/// A descendant is painted only while its ancestors are, so the walk stops at an element that
/// was not painted.
pub(crate) fn unpaint_subtree(elements: &mut ElementArena, damage: &mut Damage, id: ElementId) {
    let element = &mut elements[id];
    if !element.painted.is_painted {
        return;
    }
    damage.add(element.painted.visual_bounds);
    element.painted = PaintedElement::default();
    element.dirty.insert(Dirty::PAINT);
    for position in 0..elements[id].children.len() {
        let child = elements[id].children[position];
        unpaint_subtree(elements, damage, child);
    }
}
