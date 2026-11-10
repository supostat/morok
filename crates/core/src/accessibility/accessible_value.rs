use crate::accessibility::numeric_value::NumericValue;

/// What an element holds, as an assistive technology reads it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum AccessibleValue<'a> {
    /// Nothing of its own. A text element then gives its text.
    None,
    Text(&'a str),
    Number(NumericValue),
}
