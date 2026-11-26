use cssparser::Parser;
use morok_style::Declaration;

use crate::property_family::PropertyFamily;
use crate::value_context::ValueContext;
use crate::value_error::ValueResult;

/// Reads the value of a property into the declarations of its longhands.
pub(crate) type ValueParser = for<'i> fn(&ValueContext<'_>, &mut Parser<'i>) -> ValueResult<Vec<Declaration>>;

/// A CSS property the subset supports.
pub(crate) struct Property {
    pub(crate) name: &'static str,
    /// What the value is made of; finishes the sentence "it takes …".
    pub(crate) expectation: &'static str,
    pub(crate) parse: ValueParser,
}

pub(crate) fn named(name: &str) -> Option<&'static Property> {
    PropertyFamily::ALL
        .iter()
        .flat_map(|family| family.properties)
        .find(|property| name.eq_ignore_ascii_case(property.name))
}
