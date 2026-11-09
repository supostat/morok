/// The ID of a CSS class: an index into the classes of a [`StyleSheet`](crate::StyleSheet).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Class(u32);

impl Class {
    pub const fn new(index: u32) -> Self {
        Self(index)
    }

    pub fn index(self) -> usize {
        self.0 as usize
    }
}
