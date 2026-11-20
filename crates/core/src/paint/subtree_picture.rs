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
    /// The element of every clip of the display list, in the order of the clips.
    pub(crate) clip_owners: &'a [ElementId],
}

/// A clip or a group open around the item being read.
struct Enclosing {
    push: DisplayItem,
    /// Whether the picture is drawn inside it.
    is_kept: bool,
    /// Whether the picture holds its push already.
    is_in_picture: bool,
}

impl SubtreePicture<'_> {
    /// Writes the items of the subtree to `picture`, in their paint order and inside the
    /// groups that place them on the surface, so that they are drawn where the surface shows
    /// them. The clips of the elements inside the subtree hold, and those of the elements
    /// around it do not: the picture shows the whole subtree, the part a scrolled viewport
    /// hides included. Returns the rectangle of the surface the subtree covers; none when the
    /// display list drew nothing of it.
    pub(crate) fn write(&self, subtree: ElementId, picture: &mut DisplayList) -> Option<Rect> {
        picture.clear();
        let (mut drawn, covered) = self.items_of(subtree);
        drawn.sort_unstable_by_key(|items| items.start);
        let mut drawn = drawn.iter().peekable();
        let mut clip_owners = self.clip_owners.iter();
        let mut enclosing: Vec<Enclosing> = Vec::new();
        for (position, item) in self.display_list.items.iter().enumerate() {
            match item {
                DisplayItem::PushClip { .. } => {
                    let owner = *clip_owners
                        .next()
                        .expect("every clip of the display list has the element that clips");
                    enclosing.push(Enclosing {
                        push: *item,
                        is_kept: self.is_inside(subtree, owner),
                        is_in_picture: false,
                    });
                }
                DisplayItem::PushGroup { .. } => enclosing.push(Enclosing {
                    push: *item,
                    is_kept: true,
                    is_in_picture: false,
                }),
                DisplayItem::PopClip | DisplayItem::PopGroup => {
                    let closed = enclosing.pop().expect("a clip or a group is open when it is popped");
                    if closed.is_in_picture {
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
                    for open in &mut enclosing {
                        if open.is_kept && !open.is_in_picture {
                            picture.push_copy(open.push, self.display_list);
                            open.is_in_picture = true;
                        }
                    }
                    picture.push_copy(*item, self.display_list);
                }
            }
        }
        covered
    }

    /// Whether the element is the root of the subtree or lies inside it. An element that left
    /// the tree since the display list was written lies outside: what lay inside a subtree
    /// left with it.
    fn is_inside(&self, subtree: ElementId, element: ElementId) -> bool {
        self.elements.contains(element) && self.elements.is_inclusive_ancestor(subtree, element)
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
