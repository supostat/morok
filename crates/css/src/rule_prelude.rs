use cssparser::{Parser, Token};

use crate::span::Span;

/// What closes the prelude of a rule: the `{` of its block, a `;`, or the end of the input.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum RuleEnd {
    Block,
    Semicolon,
    EndOfInput,
}

impl RuleEnd {
    fn terminator_length(self) -> usize {
        match self {
            Self::Block | Self::Semicolon => 1,
            Self::EndOfInput => 0,
        }
    }
}

/// Reads the prelude of the next rule up to what closes it. After a `Block` end the block is
/// open: the caller reads it as a nested block or leaves it to be skipped.
pub(crate) fn read(input: &mut Parser<'_>, text: &str) -> (Span, RuleEnd) {
    input.skip_whitespace();
    let start = input.position().byte_index();
    let rule_end = loop {
        match input.next().ok().cloned() {
            Some(Token::CurlyBracketBlock) => break RuleEnd::Block,
            Some(Token::Semicolon) => break RuleEnd::Semicolon,
            Some(_) => {}
            None => break RuleEnd::EndOfInput,
        }
    };
    let prelude = Span {
        start,
        end: input.position().byte_index() - rule_end.terminator_length(),
    };
    (prelude.without_surrounding_whitespace(text), rule_end)
}
