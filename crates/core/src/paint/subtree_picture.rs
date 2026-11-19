use crate::element_arena::ElementArena;
use crate::element_id::ElementId;
use crate::geometry::rect::Rect;
use crate::paint::display_item::DisplayItem;
use crate::paint::display_list::DisplayList;
use crate::paint::item_range::ItemRange;

/// Writes what a display list drew of one element and of everything inside it, and nothing
/// else: the picture a drag shows of its source.
#[derive(Debug)]
pub(crate) struct SubtreePicture<'a> {
    pub(crate) elements: &'a ElementArena,
    pub(crate) display_list: &'a DisplayList,
}

impl SubtreePicture<'_> {
    /// Writes the items of the subtree to `picture`, in their paint order and inside the
    /// clips and the groups that hold them on the surface, so that they are drawn where the
    /// surface shows them. Returns the rectangle of the surface they cover; none when the
    /// display list drew nothing of the subtree.
    pub(crate) fn write(&self, subtree: ElementId, picture: &mut DisplayList) -> Option<Rect> {
        picture.clear();
        let (mut drawn, covered) = self.items_of(subtree);
        drawn.sort_unstable_by_key(|items| items.start);
        let mut drawn = drawn.iter().peekable();
        // The clips and the groups open around the item being read, each with whether the
        // picture holds it already.
        let mut enclosing: Vec<(DisplayItem, bool)> = Vec::new();
        for (position, item) in self.display_list.items.iter().enumerate() {
            match item {
                DisplayItem::PushClip { .. } | DisplayItem::PushGroup { .. } => enclosing.push((*item, false)),
                DisplayItem::PopClip | DisplayItem::PopGroup => {
                    let (_, is_in_picture) = enclosing.pop().expect("a clip or a group is open when it is popped");
                    if is_in_picture {
                        picture.items.push(*item);
                    }
                }
                _ => {
                    while drawn.peek().is_some_and(|items| items.end as usize <= position) {
                        drawn.next();
                    }
                    if drawn.peek().is_none_or(|items| items.start as usize > position) {
                        continue;
                    }
                    for (opened, is_in_picture) in &mut enclosing {
                        if !*is_in_picture {
                            picture.push_copy(*opened, self.display_list);
                            *is_in_picture = true;
                        }
                    }
                    picture.push_copy(*item, self.display_list);
                }
            }
        }
        covered
    }

    /// The runs of items the elements of the subtree drew, and the rectangle of the surface
    /// they cover between them.
    fn items_of(&self, subtree: ElementId) -> (Vec<ItemRange>, Option<Rect>) {
        let mut drawn = Vec::new();
        let mut covered: Option<Rect> = None;
        let mut pending = vec![subtree];
        while let Some(id) = pending.pop() {
            let element = &self.elements[id];
            pending.extend(&element.children);
            let painted = &element.painted;
            if !painted.is_painted {
                continue;
            }
            drawn.extend(
                [painted.box_items, painted.outline_items]
                    .into_iter()
                    .filter(|items| items.start != items.end),
            );
            covered = Some(covered.map_or(painted.visual_bounds, |covered| covered.union(&painted.visual_bounds)));
        }
        (drawn, covered)
    }
}
