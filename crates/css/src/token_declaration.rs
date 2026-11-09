use crate::span::Span;
use crate::token_name::TokenName;
use crate::token_value::TokenValue;

/// One `--<family>-<name>: <value>` of a tokens block.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct TokenDeclaration {
    pub(crate) name: TokenName,
    pub(crate) name_span: Span,
    /// `None` when the value was rejected; its diagnostic is already reported.
    pub(crate) value: Option<TokenValue>,
}
