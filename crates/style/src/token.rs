use std::fmt;
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;

/// The ID of a design token whose value has type `V`: an index into the table of `V` in a [`Theme`](crate::Theme).
pub struct Token<V> {
    index: u32,
    value: PhantomData<fn() -> V>,
}

impl<V> Token<V> {
    pub const fn new(index: u32) -> Self {
        Self {
            index,
            value: PhantomData,
        }
    }

    pub fn index(self) -> usize {
        self.index as usize
    }
}

impl<V> Clone for Token<V> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<V> Copy for Token<V> {}

impl<V> PartialEq for Token<V> {
    fn eq(&self, other: &Self) -> bool {
        self.index == other.index
    }
}

impl<V> Eq for Token<V> {}

impl<V> Hash for Token<V> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.index.hash(state);
    }
}

impl<V> fmt::Debug for Token<V> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.debug_tuple("Token").field(&self.index).finish()
    }
}
