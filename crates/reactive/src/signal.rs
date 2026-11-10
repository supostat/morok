use std::fmt;
use std::marker::PhantomData;

use crate::node_id::NodeId;

/// A handle to a value that is written from outside the graph and read by memos and effects.
pub struct Signal<T> {
    pub(crate) id: NodeId,
    value: PhantomData<fn() -> T>,
}

impl<T> Signal<T> {
    pub(crate) const fn new(id: NodeId) -> Self {
        Self { id, value: PhantomData }
    }
}

impl<T> Clone for Signal<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for Signal<T> {}

impl<T> PartialEq for Signal<T> {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl<T> Eq for Signal<T> {}

impl<T> fmt::Debug for Signal<T> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.debug_tuple("Signal").field(&self.id).finish()
    }
}
