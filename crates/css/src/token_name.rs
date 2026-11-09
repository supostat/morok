use std::fmt;

use crate::token_family::TokenFamily;
use crate::token_kind::TokenKind;
use crate::token_name_error::TokenNameError;

const CUSTOM_PROPERTY_PREFIX: &str = "--";

/// A token name without the leading `--`, known to be `<family>-<name>` in lowercase kebab-case.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(crate) struct TokenName {
    text: String,
    family: TokenFamily,
}

impl TokenName {
    pub(crate) fn is_custom_property(property: &str) -> bool {
        property.starts_with(CUSTOM_PROPERTY_PREFIX)
    }

    pub(crate) fn parse(custom_property: &str) -> Result<Self, TokenNameError> {
        let text = custom_property.trim_start_matches(CUSTOM_PROPERTY_PREFIX);
        if !is_lowercase_kebab_case(text) {
            return Err(TokenNameError::NotLowercaseKebabCase);
        }
        let (family_name, name) = match text.split_once('-') {
            Some((family_name, name)) => (family_name, Some(name)),
            None => (text, None),
        };
        let Some(family) = TokenFamily::named(family_name) else {
            return Err(TokenNameError::UnknownFamily {
                family: family_name.to_owned(),
            });
        };
        if name.is_none() {
            return Err(TokenNameError::NameMissing);
        }
        Ok(Self {
            text: text.to_owned(),
            family,
        })
    }

    pub(crate) fn into_string(self) -> String {
        self.text
    }

    pub(crate) fn kind(&self) -> TokenKind {
        TokenKind::of(self.family, &self.text)
    }
}

impl fmt::Display for TokenName {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{CUSTOM_PROPERTY_PREFIX}{}", self.text)
    }
}

fn is_lowercase_kebab_case(text: &str) -> bool {
    text.split('-').all(|word| {
        !word.is_empty()
            && word
                .bytes()
                .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit())
    })
}
