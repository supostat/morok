use crate::text::font_request::FontRequest;
use crate::text::width_constraint::WidthConstraint;

/// One question to a [`TextShaper`](crate::TextShaper): a text, how it is set and the width
/// its lines may take.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TextRequest<'a> {
    /// The text after white space processing: a line break in it always ends a line.
    pub text: &'a str,
    pub font: FontRequest<'a>,
    /// The space added after every character, in CSS pixels.
    pub letter_spacing: f32,
    pub width: WidthConstraint,
}
