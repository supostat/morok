use crate::compiled_tokens::CompiledTokens;
use crate::keyframes_table::KeyframesTable;

/// What a value may refer to beyond its own text: the tokens and the `@keyframes` rules of the
/// stylesheets compiled together.
#[derive(Clone, Copy, Debug)]
pub(crate) struct ValueContext<'a> {
    pub(crate) tokens: &'a CompiledTokens,
    pub(crate) keyframes: &'a KeyframesTable,
}
