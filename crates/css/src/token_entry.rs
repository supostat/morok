use crate::token_id::TokenId;

/// A compiled token: its name without the leading `--` and its typed ID.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TokenEntry {
    pub name: String,
    pub id: TokenId,
}
