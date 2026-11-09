use morok_style::Theme;

use crate::token_entry::TokenEntry;
use crate::token_id::TokenId;

/// The result of compiling a tokens file: every token in declaration order and both theme tables.
#[derive(Clone, Debug, PartialEq)]
pub struct CompiledTokens {
    pub entries: Vec<TokenEntry>,
    pub light: Theme,
    pub dark: Theme,
}

impl CompiledTokens {
    /// Looks a token up by its name without the leading `--`.
    pub fn id(&self, token_name: &str) -> Option<TokenId> {
        self.entries
            .iter()
            .find(|entry| entry.name == token_name)
            .map(|entry| entry.id)
    }
}
