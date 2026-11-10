use crate::accessibility::accessible_value::AccessibleValue;
use crate::accessibility::numeric_value::NumericValue;

/// The value of an element as the tree keeps it.
#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) enum StoredValue {
    #[default]
    None,
    Text(String),
    Number(NumericValue),
}

impl StoredValue {
    pub(crate) fn is(&self, value: AccessibleValue<'_>) -> bool {
        match (self, value) {
            (Self::None, AccessibleValue::None) => true,
            (Self::Text(held), AccessibleValue::Text(text)) => held == text,
            (Self::Number(held), AccessibleValue::Number(number)) => *held == number,
            _ => false,
        }
    }
}

impl From<AccessibleValue<'_>> for StoredValue {
    fn from(value: AccessibleValue<'_>) -> Self {
        match value {
            AccessibleValue::None => Self::None,
            AccessibleValue::Text(text) => Self::Text(text.to_owned()),
            AccessibleValue::Number(number) => Self::Number(number),
        }
    }
}
