use morok_core::ElementId;
use morok_reactive::Scope;

/// One child of a keyed fragment: the element a component built for a key, and the scope that
/// owns what the component created.
#[derive(Debug)]
pub(crate) struct MountedItem<Key> {
    pub(crate) key: Key,
    pub(crate) scope: Scope,
    pub(crate) element: ElementId,
}
