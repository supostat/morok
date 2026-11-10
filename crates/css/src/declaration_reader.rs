use cssparser::{Delimiter, ParseError, Parser, Token};
use morok_style::Declaration;

use crate::diagnostic::Diagnostic;
use crate::problem::Problem;
use crate::property;
use crate::pseudo_element::PseudoElement;
use crate::source_file::SourceFile;
use crate::span::Span;
use crate::value_context::ValueContext;
use crate::value_error::ValueError;

const CUSTOM_PROPERTY_PREFIX: &str = "--";

/// What holds a declaration list, and so which properties it may set.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum DeclarationBlock {
    /// The block of a class rule: every supported property.
    Rule,
    /// The block of a keyframe: the animatable properties only.
    Keyframe,
    /// The block of a rule of a pseudo-element: the properties the pseudo-element takes, read
    /// into the declarations of its element that carry them.
    PseudoElement(PseudoElement),
}

/// Reads the `property: value;` list of a block into longhand declarations.
pub(crate) fn read(
    source: &SourceFile,
    context: &ValueContext<'_>,
    block: DeclarationBlock,
    input: &mut Parser<'_>,
    diagnostics: &mut Vec<Diagnostic>,
) -> Vec<Declaration> {
    let mut declarations = Vec::new();
    loop {
        input.skip_whitespace();
        let start = input.position();
        let Some(token) = input.next().ok().cloned() else {
            return declarations;
        };
        let name_span = Span::between(start, input.position());
        let outcome = match token {
            Token::Semicolon => continue,
            Token::Ident(name) => declaration(source, context, block, input, &name, name_span),
            _ => {
                skip_to_declaration_end(input);
                Err(Problem {
                    span: name_span,
                    message: "expected a declaration".to_owned(),
                    hint: "write it as `property: value;`".to_owned(),
                })
            }
        };
        match outcome {
            Ok(longhands) => declarations.extend(longhands),
            Err(problem) => diagnostics.push(Diagnostic::new(source, problem.span, problem.message, problem.hint)),
        }
    }
}

fn declaration(
    source: &SourceFile,
    context: &ValueContext<'_>,
    block: DeclarationBlock,
    input: &mut Parser<'_>,
    name: &str,
    name_span: Span,
) -> Result<Vec<Declaration>, Problem> {
    if input.expect_colon().is_err() {
        skip_to_declaration_end(input);
        return Err(Problem {
            span: name_span,
            message: format!("`{name}` has no value"),
            hint: format!("write `{name}: <value>;`"),
        });
    }
    input
        .parse_until_before(Delimiter::Semicolon, |value_input| {
            let outcome = value(source, context, block, value_input, name, name_span);
            skip_to_end(value_input);
            Ok::<_, ParseError<()>>(outcome)
        })
        .expect("a declaration value is read to its end")
}

fn value(
    source: &SourceFile,
    context: &ValueContext<'_>,
    block: DeclarationBlock,
    input: &mut Parser<'_>,
    name: &str,
    name_span: Span,
) -> Result<Vec<Declaration>, Problem> {
    input.skip_whitespace();
    let value_start = input.state();
    let mut important_start = None;
    loop {
        input.skip_whitespace();
        let token_start = input.position();
        match input.next() {
            Ok(Token::Delim('!')) => {
                important_start.get_or_insert(token_start);
            }
            Ok(_) => {}
            Err(_) => break,
        }
    }
    let value_span =
        Span::between(value_start.position(), input.position()).without_surrounding_whitespace(source.text());
    input.reset(&value_start);
    if let Some(important_start) = important_start {
        return Err(Problem {
            span: Span {
                start: important_start.byte_index(),
                end: value_span.end,
            },
            message: "`!important` is not supported".to_owned(),
            hint: "raise the specificity of the selector instead".to_owned(),
        });
    }
    if name.starts_with(CUSTOM_PROPERTY_PREFIX) {
        return Err(Problem {
            span: name_span,
            message: format!("`{name}` is declared in a component stylesheet"),
            hint: "tokens are declared in the tokens file".to_owned(),
        });
    }
    let Some(property) = property::named(name) else {
        return Err(Problem {
            span: name_span,
            message: format!("`{name}` is not a supported property"),
            hint: "remove it or express it through the supported CSS subset".to_owned(),
        });
    };
    let parsed = (property.parse)(context, input).and_then(|longhands| {
        if input.is_exhausted() {
            Ok(longhands)
        } else {
            Err(ValueError::Mismatch)
        }
    });
    let longhands = parsed.map_err(|error| match error {
        ValueError::Problem(problem) => problem,
        ValueError::Mismatch => Problem {
            span: value_span,
            message: format!("`{}` does not support this value", property.name),
            hint: format!("it takes {}", property.expectation),
        },
    })?;
    let is_animatable = longhands
        .iter()
        .all(|longhand| longhand.animatable_property().is_some());
    if block == DeclarationBlock::Keyframe && !is_animatable {
        return Err(Problem {
            span: name_span,
            message: format!("`{}` cannot be animated", property.name),
            hint: "a keyframe sets lengths, colors, `box-shadow`, `opacity` and `transform`".to_owned(),
        });
    }
    let DeclarationBlock::PseudoElement(pseudo_element) = block else {
        return Ok(longhands);
    };
    longhands
        .into_iter()
        .map(|longhand| pseudo_element.declaration(longhand))
        .collect::<Option<Vec<_>>>()
        .ok_or_else(|| Problem {
            span: name_span,
            message: format!("`{}` is not supported in `::{}`", property.name, pseudo_element.name()),
            hint: format!("it takes {}", pseudo_element.expectation()),
        })
}

fn skip_to_end(input: &mut Parser<'_>) {
    while input.next().is_ok() {}
}

fn skip_to_declaration_end(input: &mut Parser<'_>) {
    input
        .parse_until_before(Delimiter::Semicolon, |rest| {
            skip_to_end(rest);
            Ok::<(), ParseError<()>>(())
        })
        .expect("the rest of a declaration is read to its end");
}
