pub(crate) mod color;
pub(crate) mod font_families;
pub(crate) mod length;
pub(crate) mod motion;
pub(crate) mod number;
pub(crate) mod shadow;

use cssparser::ParseError;

pub(crate) type Parsed<T> = Result<T, ParseError<()>>;

pub(crate) fn rejected<T>() -> Parsed<T> {
    Err(ParseError::custom(()))
}
