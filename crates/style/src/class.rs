/// The ID of a CSS class: an index into the classes of a [`StyleSheet`](crate::StyleSheet).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Class(u32);

impl Class {
    /// The class at `index` among the classes of a style sheet: the compiler of the stylesheets
    /// gives every class its index.
    pub const fn new(index: u32) -> Self {
        Self(index)
    }

    /// The place of the class among the classes of its style sheet.
    pub fn index(self) -> usize {
        self.0 as usize
    }
}
