use crate::layout::width_constraint::WidthConstraint;

/// One question to a [`TextMeasurer`](crate::TextMeasurer): the text of an element and the width it may take.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TextMeasureRequest<'a> {
    pub text: &'a str,
    pub width: WidthConstraint,
}
