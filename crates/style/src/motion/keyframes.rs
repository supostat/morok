/// The ID of a `@keyframes` rule: an index into the keyframes of a [`StyleSheet`](crate::StyleSheet).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Keyframes(u32);

impl Keyframes {
    /// The `@keyframes` rule at `index` among those of a style sheet: the compiler of the
    /// stylesheets gives every rule its index.
    pub const fn new(index: u32) -> Self {
        Self(index)
    }

    /// The place of the rule among the `@keyframes` rules of its style sheet.
    pub fn index(self) -> usize {
        self.0 as usize
    }
}
