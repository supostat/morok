use morok_core::ElementId;
use morok_reactive::{Runtime, Scope};

use crate::view::fragment_id::FragmentId;
use crate::view::mount::Mount;
use crate::view::mounted_item::MountedItem;
use crate::view::surface_id::SurfaceId;
use crate::view::surfaces::Surfaces;

/// A fragment whose children are told apart by keys: a key that stays keeps its element, a key
/// that comes gets an element mounted for it, and a key that goes takes its element along.
#[derive(Debug)]
pub(crate) struct KeyedFragment<Key> {
    pub(crate) surface: SurfaceId,
    pub(crate) fragment: FragmentId,
    /// Owns the scopes of the items, so that they outlive the runs of the effect that mounts
    /// them and go with whatever mounted the fragment.
    pub(crate) scope: Scope,
    pub(crate) items: Vec<MountedItem<Key>>,
}

impl<Key: PartialEq> KeyedFragment<Key> {
    /// Brings the children of the fragment in line with `keys`, which it empties: `mount_item`
    /// builds the element of a key that came, given the place of the key among the keys.
    pub(crate) fn reconcile(
        &mut self,
        runtime: &mut Runtime<Surfaces>,
        surfaces: &mut Surfaces,
        keys: &mut Vec<Key>,
        mount_item: &dyn Fn(&mut Mount<'_>, usize) -> ElementId,
    ) {
        let surface = surfaces.surface_mut(self.surface);
        let mut position = 0;
        while position < self.items.len() {
            if keys.contains(&self.items[position].key) {
                position += 1;
            } else {
                let gone = self.items.remove(position);
                runtime.dispose(gone.scope);
                surface.remove_subtree(gone.element);
            }
        }

        let parent = self.fragment.parent;
        let start = surface.child_fragments.start(self.fragment);
        let key_count = keys.len();
        for (position, key) in keys.drain(..).enumerate() {
            let offset_of_mounted = self.items[position..].iter().position(|item| item.key == key);
            match offset_of_mounted {
                Some(0) => {}
                Some(offset) => {
                    let moved = self.items.remove(position + offset);
                    surface.tree.detach(moved.element);
                    surface.tree.insert_child(parent, start + position, moved.element);
                    self.items.insert(position, moved);
                }
                None => {
                    let (scope, element) = Mount::in_scope(runtime, surface, self.surface, self.scope, |mount| {
                        mount_item(mount, position)
                    });
                    surface.tree.insert_child(parent, start + position, element);
                    self.items.insert(position, MountedItem { key, scope, element });
                }
            }
        }
        // Items mounted for a key the keys held twice and now hold once.
        while self.items.len() > key_count {
            let gone = self.items.pop().expect("there are more items than keys");
            runtime.dispose(gone.scope);
            surface.remove_subtree(gone.element);
        }
        surface.child_fragments.set_length(self.fragment, self.items.len());
    }
}
