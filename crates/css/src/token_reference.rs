use cssparser::{Parser, Token};
use morok_style::{Color, Pixels, Shadow, Token as StyleToken};

use crate::compiled_tokens::CompiledTokens;
use crate::problem::Problem;
use crate::property_syntax::{nested, peek};
use crate::span::Span;
use crate::token_id::TokenId;
use crate::value_error::{ValueError, ValueResult};

const VAR: &str = "var";
const CUSTOM_PROPERTY_PREFIX: &str = "--";

/// A `var(--name)` that names a token of the compiled tokens file.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct TokenReference {
    name: String,
    id: TokenId,
    span: Span,
}

pub(crate) fn is_ahead(input: &mut Parser<'_>) -> bool {
    matches!(peek(input), Some(Token::Function(name)) if name.eq_ignore_ascii_case(VAR))
}

/// The token the `var()` ahead refers to, without consuming it; `None` when no `var()` is ahead.
pub(crate) fn id_ahead(tokens: &CompiledTokens, input: &mut Parser<'_>) -> ValueResult<Option<TokenId>> {
    if !is_ahead(input) {
        return Ok(None);
    }
    let before = input.state();
    let reference = parse(tokens, input);
    input.reset(&before);
    Ok(Some(reference?.id))
}

pub(crate) fn parse(tokens: &CompiledTokens, input: &mut Parser<'_>) -> ValueResult<TokenReference> {
    input.skip_whitespace();
    let start = input.position();
    if !matches!(input.next(), Ok(Token::Function(name)) if name.eq_ignore_ascii_case(VAR)) {
        return Err(ValueError::Mismatch);
    }
    let (custom_property, has_fallback) = nested(input, |arguments| {
        let custom_property = arguments.expect_ident_cloned().map_err(|_| ValueError::Mismatch)?;
        let has_fallback = !arguments.is_exhausted();
        while arguments.next().is_ok() {}
        Ok((custom_property.as_ref().to_owned(), has_fallback))
    })?;
    let span = Span::between(start, input.position());
    if has_fallback {
        return Err(Problem {
            span,
            message: "a `var()` fallback is not supported".to_owned(),
            hint: "every token is defined in the tokens file: drop the fallback".to_owned(),
        }
        .into());
    }
    let Some(name) = custom_property.strip_prefix(CUSTOM_PROPERTY_PREFIX) else {
        return Err(ValueError::Mismatch);
    };
    let Some(id) = tokens.id(name) else {
        return Err(Problem {
            span,
            message: format!("unknown token `{custom_property}`"),
            hint: "define it in the tokens file".to_owned(),
        }
        .into());
    };
    Ok(TokenReference {
        name: name.to_owned(),
        id,
        span,
    })
}

impl TokenReference {
    pub(crate) fn length(&self) -> ValueResult<StyleToken<Pixels>> {
        match self.id {
            TokenId::Length(token) => Ok(token),
            _ => Err(self.is_not("a length")),
        }
    }

    pub(crate) fn color(&self) -> ValueResult<StyleToken<Color>> {
        match self.id {
            TokenId::Color(token) => Ok(token),
            _ => Err(self.is_not("a color")),
        }
    }

    pub(crate) fn shadow(&self) -> ValueResult<StyleToken<Shadow>> {
        match self.id {
            TokenId::Shadow(token) => Ok(token),
            _ => Err(self.is_not("a shadow")),
        }
    }

    fn is_not(&self, expected: &str) -> ValueError {
        Problem {
            span: self.span,
            message: format!(
                "`{CUSTOM_PROPERTY_PREFIX}{}` is {}, not {expected}",
                self.name,
                self.id.description()
            ),
            hint: format!("use a token that holds {expected}"),
        }
        .into()
    }
}
