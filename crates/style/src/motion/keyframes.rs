/// The ID of a `@keyframes` rule: an index into the keyframes of a [`StyleSheet`](crate::StyleSheet).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Keyframes(u32);

impl Keyframes {
    pub const fn new(index: u32) -> Self {
        Self(index)
    }

    pub fn index(self) -> usize {
        self.0 as usize
    }
}
