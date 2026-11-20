use crate::span::Span;
use crate::token_declaration::TokenDeclaration;

/// The `:root` block of one scheme of a tokens file.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct TokensBlock {
    /// The `:root` selector of the block.
    pub(crate) root: Span,
    pub(crate) declarations: Vec<TokenDeclaration>,
}
