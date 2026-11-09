use std::borrow::Cow;

/// Font family names in order of preference.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct FontFamilies {
    pub names: Cow<'static, [Cow<'static, str>]>,
}
