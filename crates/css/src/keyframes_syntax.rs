use cssparser::{Delimiter, Parser, Token};

use crate::problem::Problem;
use crate::span::Span;
use crate::value_syntax::rejected;

const AT_KEYFRAMES: &str = "@keyframes";
/// The words that cannot name keyframes: CSS-wide keywords and the keywords an `animation`
/// declaration reads before a name.
const RESERVED_NAMES: [&str; 16] = [
    "none",
    "initial",
    "inherit",
    "unset",
    "default",
    "linear",
    "infinite",
    "normal",
    "reverse",
    "alternate",
    "alternate-reverse",
    "forwards",
    "backwards",
    "both",
    "running",
    "paused",
];

/// The name of a `@keyframes` rule and where it is written.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct KeyframesName {
    pub(crate) text: String,
    pub(crate) span: Span,
}

/// The name of the `@keyframes` rule whose prelude this is; none when the prelude opens another
/// rule.
pub(crate) fn name(text: &str, prelude: Span) -> Option<Result<KeyframesName, Problem>> {
    let prelude_text = &text[prelude.start..prelude.end];
    let keyword = prelude_text.get(..AT_KEYFRAMES.len())?;
    let rest = &prelude_text[AT_KEYFRAMES.len()..];
    if !keyword.eq_ignore_ascii_case(AT_KEYFRAMES) || !rest.starts_with(char::is_whitespace) {
        return None;
    }
    let name_span = Span {
        start: prelude.start + AT_KEYFRAMES.len(),
        end: prelude.end,
    }
    .without_surrounding_whitespace(text);
    let name_text = &text[name_span.start..name_span.end];
    let mut input = Parser::new(name_text);
    let token = input.next().ok().cloned();
    let ident = match token {
        Some(Token::Ident(ident)) if input.is_exhausted() => ident.as_ref().to_owned(),
        _ => {
            return Some(Err(Problem {
                span: name_span,
                message: format!("`{name_text}` is not a name of keyframes"),
                hint: "name the keyframes with one identifier: `@keyframes fade-in`".to_owned(),
            }));
        }
    };
    if RESERVED_NAMES
        .iter()
        .any(|reserved| ident.eq_ignore_ascii_case(reserved))
    {
        return Some(Err(Problem {
            span: name_span,
            message: format!("`{ident}` cannot name keyframes"),
            hint: "an `animation` declaration reads this word as a keyword: choose another name".to_owned(),
        }));
    }
    Some(Ok(KeyframesName {
        text: ident,
        span: name_span,
    }))
}

/// The offsets a keyframe prelude lists: `from`, `to` and percentages, separated by commas.
pub(crate) fn offsets(text: &str, prelude: Span) -> Result<Vec<f32>, Problem> {
    let mut input = Parser::new(&text[prelude.start..prelude.end]);
    let mut offsets = Vec::new();
    loop {
        input.skip_whitespace();
        let start = input.position();
        let offset = input.parse_until_before(Delimiter::Comma, |item| {
            let offset = keyframe_offset(item.next().ok());
            match offset {
                Some(offset) if item.is_exhausted() => Ok(offset),
                _ => rejected(),
            }
        });
        let Ok(offset) = offset else {
            let span = Span {
                start: prelude.start + start.byte_index(),
                end: prelude.start + input.position().byte_index(),
            }
            .without_surrounding_whitespace(text);
            return Err(Problem {
                span,
                message: format!("`{}` is not a keyframe selector", &text[span.start..span.end]),
                hint: "a keyframe is `from`, `to` or a percentage from 0% to 100%".to_owned(),
            });
        };
        offsets.push(offset);
        if input.next().is_err() {
            return Ok(offsets);
        }
    }
}

fn keyframe_offset(token: Option<&Token<'_>>) -> Option<f32> {
    match token? {
        Token::Ident(ident) if ident.eq_ignore_ascii_case("from") => Some(0.0),
        Token::Ident(ident) if ident.eq_ignore_ascii_case("to") => Some(1.0),
        Token::Percentage { unit_value, .. } if (0.0..=1.0).contains(unit_value) => Some(*unit_value),
        _ => None,
    }
}
