use cssparser::{Parser, Token};
use morok_style::{BackgroundImage, BoxShadow, ColorValue, Declaration};

use crate::compiled_tokens::CompiledTokens;
use crate::problem::Problem;
use crate::property::Property;
use crate::property_syntax::{accepts_keyword, color, gradient, peek};
use crate::span::Span;
use crate::token_reference;
use crate::value_error::{ValueError, ValueResult};

const NONE: &str = "none";
const URL: &str = "url";

const COLOR: &str = "a color token, `transparent` or `currentColor`";
const IMAGE: &str = "`none`, `linear-gradient()` or `radial-gradient()`";

pub(crate) const PROPERTIES: &[Property] = &[
    Property {
        name: "background",
        expectation: "`none`, a color, a gradient or both",
        parse: background,
    },
    Property {
        name: "background-color",
        expectation: COLOR,
        parse: |tokens, input| Ok(vec![Declaration::BackgroundColor(color::parse(tokens, input)?)]),
    },
    Property {
        name: "background-image",
        expectation: IMAGE,
        parse: |tokens, input| Ok(vec![Declaration::BackgroundImage(image(tokens, input)?)]),
    },
    Property {
        name: "box-shadow",
        expectation: "`none` or a shadow token",
        parse: |tokens, input| Ok(vec![Declaration::BoxShadow(box_shadow(tokens, input)?)]),
    },
    Property {
        name: "color",
        expectation: COLOR,
        parse: |tokens, input| Ok(vec![Declaration::Color(color::parse(tokens, input)?)]),
    },
];

/// `background`: a color and an image in either order; an omitted one takes its initial value.
fn background(tokens: &CompiledTokens, input: &mut Parser<'_>) -> ValueResult<Vec<Declaration>> {
    let mut background_color = None;
    let mut background_image = None;
    while !input.is_exhausted() {
        input.skip_whitespace();
        let start = input.position();
        if let Some(Token::Comma) = peek(input) {
            while input.next().is_ok() {}
            return Err(Problem {
                span: Span::between(start, input.position()),
                message: "several background layers are not supported".to_owned(),
                hint: "keep one gradient and one color".to_owned(),
            }
            .into());
        }
        if background_image.is_none() && image_is_ahead(input) {
            background_image = Some(image(tokens, input)?);
        } else if background_color.is_none() {
            background_color = Some(color::parse(tokens, input)?);
        } else {
            return Err(ValueError::Mismatch);
        }
    }
    if background_color.is_none() && background_image.is_none() {
        return Err(ValueError::Mismatch);
    }
    Ok(vec![
        Declaration::BackgroundColor(background_color.unwrap_or(ColorValue::Transparent)),
        Declaration::BackgroundImage(background_image.unwrap_or(BackgroundImage::None)),
    ])
}

fn image_is_ahead(input: &mut Parser<'_>) -> bool {
    gradient::is_ahead(input)
        || match peek(input) {
            Some(Token::Ident(name)) => name.eq_ignore_ascii_case(NONE),
            Some(Token::UnquotedUrl(_)) => true,
            Some(Token::Function(name)) => name.eq_ignore_ascii_case(URL),
            _ => false,
        }
}

fn image(tokens: &CompiledTokens, input: &mut Parser<'_>) -> ValueResult<BackgroundImage> {
    if accepts_keyword(input, NONE) {
        return Ok(BackgroundImage::None);
    }
    input.skip_whitespace();
    let start = input.position();
    let is_url = match peek(input) {
        Some(Token::UnquotedUrl(_)) => true,
        Some(Token::Function(name)) => name.eq_ignore_ascii_case(URL),
        _ => false,
    };
    if is_url {
        while input.next().is_ok() {}
        return Err(Problem {
            span: Span::between(start, input.position()),
            message: "`url()` is not supported".to_owned(),
            hint: "an image is drawn by the `Image` element".to_owned(),
        }
        .into());
    }
    Ok(BackgroundImage::Gradient(gradient::parse(tokens, input)?))
}

fn box_shadow(tokens: &CompiledTokens, input: &mut Parser<'_>) -> ValueResult<BoxShadow> {
    if accepts_keyword(input, NONE) {
        return Ok(BoxShadow::None);
    }
    if token_reference::is_ahead(input) {
        return Ok(BoxShadow::Token(token_reference::parse(tokens, input)?.shadow()?));
    }
    input.skip_whitespace();
    let start = input.position();
    if input.is_exhausted() {
        return Err(ValueError::Mismatch);
    }
    while input.next().is_ok() {}
    Err(Problem {
        span: Span::between(start, input.position()),
        message: "raw shadows are not supported".to_owned(),
        hint: "a shadow comes from a `shadow-` token: `var(--shadow-…)`".to_owned(),
    }
    .into())
}
