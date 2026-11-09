use std::borrow::Cow;

/// A class with the rules it keys: those whose subject names this class first.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ClassStyle {
    pub name: Cow<'static, str>,
    /// Ascending indices into [`StyleSheet::rules`](crate::StyleSheet::rules).
    pub rules: Cow<'static, [u32]>,
}
