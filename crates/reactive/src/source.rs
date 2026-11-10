use std::fmt;
use std::marker::PhantomData;

use crate::node_id::NodeId;

/// A handle to a value that can be read: a [`Signal`](crate::Signal) or a [`Memo`](crate::Memo).
pub struct Source<T> {
    pub(crate) id: NodeId,
    value: PhantomData<fn() -> T>,
}

impl<T> Source<T> {
    pub(crate) const fn new(id: NodeId) -> Self {
        Self { id, value: PhantomData }
    }
}

impl<T> Clone for Source<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for Source<T> {}

impl<T> PartialEq for Source<T> {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl<T> Eq for Source<T> {}

impl<T> fmt::Debug for Source<T> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.debug_tuple("Source").field(&self.id).finish()
    }
}

impl<T> From<crate::Signal<T>> for Source<T> {
    fn from(signal: crate::Signal<T>) -> Self {
        Self::new(signal.id)
    }
}

impl<T> From<crate::Memo<T>> for Source<T> {
    fn from(memo: crate::Memo<T>) -> Self {
        Self::new(memo.id)
    }
}
