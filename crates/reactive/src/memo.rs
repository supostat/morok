use std::fmt;
use std::marker::PhantomData;

use crate::node_id::NodeId;

/// A handle to a value computed from signals and other memos, recomputed only after one of the
/// values it read last time changed.
pub struct Memo<T> {
    pub(crate) id: NodeId,
    value: PhantomData<fn() -> T>,
}

impl<T> Memo<T> {
    pub(crate) const fn new(id: NodeId) -> Self {
        Self { id, value: PhantomData }
    }
}

impl<T> Clone for Memo<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for Memo<T> {}

impl<T> PartialEq for Memo<T> {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl<T> Eq for Memo<T> {}

impl<T> fmt::Debug for Memo<T> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.debug_tuple("Memo").field(&self.id).finish()
    }
}
