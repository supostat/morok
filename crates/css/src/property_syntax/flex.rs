use cssparser::Parser;
use morok_style::{
    ContentAlignment, Declaration, FlexDirection, FlexWrap, ItemAlignment, LengthPercentage, LengthPercentageAuto,
    Pixels, SelfAlignment,
};
use std::borrow::Cow;

use crate::compiled_tokens::CompiledTokens;
use crate::property::Property;
use crate::property_syntax::layout::size;
use crate::property_syntax::{accepts_keyword, keyword, non_negative_number};
use crate::value_error::{ValueError, ValueResult};

const DIRECTIONS: [(&str, FlexDirection); 4] = [
    ("row", FlexDirection::Row),
    ("row-reverse", FlexDirection::RowReverse),
    ("column", FlexDirection::Column),
    ("column-reverse", FlexDirection::ColumnReverse),
];
const WRAPS: [(&str, FlexWrap); 3] = [
    ("nowrap", FlexWrap::NoWrap),
    ("wrap", FlexWrap::Wrap),
    ("wrap-reverse", FlexWrap::WrapReverse),
];
const CONTENT_ALIGNMENTS: [(&str, ContentAlignment); 10] = [
    ("normal", ContentAlignment::Normal),
    ("start", ContentAlignment::Start),
    ("end", ContentAlignment::End),
    ("flex-start", ContentAlignment::FlexStart),
    ("flex-end", ContentAlignment::FlexEnd),
    ("center", ContentAlignment::Center),
    ("stretch", ContentAlignment::Stretch),
    ("space-between", ContentAlignment::SpaceBetween),
    ("space-around", ContentAlignment::SpaceAround),
    ("space-evenly", ContentAlignment::SpaceEvenly),
];
const ITEM_ALIGNMENTS: [(&str, ItemAlignment); 8] = [
    ("normal", ItemAlignment::Normal),
    ("start", ItemAlignment::Start),
    ("end", ItemAlignment::End),
    ("flex-start", ItemAlignment::FlexStart),
    ("flex-end", ItemAlignment::FlexEnd),
    ("center", ItemAlignment::Center),
    ("baseline", ItemAlignment::Baseline),
    ("stretch", ItemAlignment::Stretch),
];

const AUTO: &str = "auto";
const NONE: &str = "none";

const DIRECTION: &str = "`row`, `row-reverse`, `column` or `column-reverse`";
const WRAP: &str = "`nowrap`, `wrap` or `wrap-reverse`";
const FACTOR: &str = "a number, not negative";
const CONTENT_ALIGNMENT: &str = "`normal`, `start`, `end`, `flex-start`, `flex-end`, `center`, `stretch`, \
                                 `space-between`, `space-around` or `space-evenly`";
const ITEM_ALIGNMENT: &str = "`normal`, `start`, `end`, `flex-start`, `flex-end`, `center`, `baseline` or `stretch`";
const SELF_ALIGNMENT: &str =
    "`auto`, `normal`, `start`, `end`, `flex-start`, `flex-end`, `center`, `baseline` or `stretch`";

pub(crate) const PROPERTIES: &[Property] = &[
    Property {
        name: "flex-direction",
        expectation: DIRECTION,
        parse: |_, input| Ok(vec![Declaration::FlexDirection(keyword(input, &DIRECTIONS)?)]),
    },
    Property {
        name: "flex-wrap",
        expectation: WRAP,
        parse: |_, input| Ok(vec![Declaration::FlexWrap(keyword(input, &WRAPS)?)]),
    },
    Property {
        name: "flex-flow",
        expectation: "a flex direction, a flex wrap or both",
        parse: |_, input| flow(input),
    },
    Property {
        name: "flex-grow",
        expectation: FACTOR,
        parse: |_, input| Ok(vec![Declaration::FlexGrow(non_negative_number(input)?)]),
    },
    Property {
        name: "flex-shrink",
        expectation: FACTOR,
        parse: |_, input| Ok(vec![Declaration::FlexShrink(non_negative_number(input)?)]),
    },
    Property {
        name: "flex-basis",
        expectation: "a length, a percentage, `calc()` or `auto`",
        parse: |context, input| Ok(vec![Declaration::FlexBasis(size(context.tokens, input)?)]),
    },
    Property {
        name: "flex",
        expectation: "`none`, or a grow factor, a shrink factor and a basis",
        parse: |context, input| flex(context.tokens, input),
    },
    Property {
        name: "justify-content",
        expectation: CONTENT_ALIGNMENT,
        parse: |_, input| Ok(vec![Declaration::JustifyContent(keyword(input, &CONTENT_ALIGNMENTS)?)]),
    },
    Property {
        name: "align-content",
        expectation: CONTENT_ALIGNMENT,
        parse: |_, input| Ok(vec![Declaration::AlignContent(keyword(input, &CONTENT_ALIGNMENTS)?)]),
    },
    Property {
        name: "justify-items",
        expectation: ITEM_ALIGNMENT,
        parse: |_, input| Ok(vec![Declaration::JustifyItems(keyword(input, &ITEM_ALIGNMENTS)?)]),
    },
    Property {
        name: "align-items",
        expectation: ITEM_ALIGNMENT,
        parse: |_, input| Ok(vec![Declaration::AlignItems(keyword(input, &ITEM_ALIGNMENTS)?)]),
    },
    Property {
        name: "justify-self",
        expectation: SELF_ALIGNMENT,
        parse: |_, input| Ok(vec![Declaration::JustifySelf(self_alignment(input)?)]),
    },
    Property {
        name: "align-self",
        expectation: SELF_ALIGNMENT,
        parse: |_, input| Ok(vec![Declaration::AlignSelf(self_alignment(input)?)]),
    },
];

fn self_alignment(input: &mut Parser<'_>) -> ValueResult<SelfAlignment> {
    if accepts_keyword(input, AUTO) {
        return Ok(SelfAlignment::Auto);
    }
    Ok(SelfAlignment::Item(keyword(input, &ITEM_ALIGNMENTS)?))
}

/// `flex-flow`: the direction and the wrap in either order; an omitted one takes its initial value.
fn flow(input: &mut Parser<'_>) -> ValueResult<Vec<Declaration>> {
    let mut direction = None;
    let mut wrap = None;
    while !input.is_exhausted() {
        if direction.is_none()
            && let Ok(parsed) = input.try_parse(|input| keyword(input, &DIRECTIONS))
        {
            direction = Some(parsed);
        } else if wrap.is_none()
            && let Ok(parsed) = input.try_parse(|input| keyword(input, &WRAPS))
        {
            wrap = Some(parsed);
        } else {
            return Err(ValueError::Mismatch);
        }
    }
    if direction.is_none() && wrap.is_none() {
        return Err(ValueError::Mismatch);
    }
    Ok(vec![
        Declaration::FlexDirection(direction.unwrap_or(FlexDirection::Row)),
        Declaration::FlexWrap(wrap.unwrap_or(FlexWrap::NoWrap)),
    ])
}

/// `flex`: `none`, or the two factors together and the basis in either order. An omitted factor
/// is 1 and an omitted basis is `0%`, as in CSS.
fn flex(tokens: &CompiledTokens, input: &mut Parser<'_>) -> ValueResult<Vec<Declaration>> {
    if accepts_keyword(input, NONE) {
        return Ok(longhands(0.0, 0.0, LengthPercentageAuto::Auto));
    }
    let mut grow = None;
    let mut shrink = None;
    let mut basis = None;
    let mut factors_are_closed = false;
    while !input.is_exhausted() {
        if let Ok(number) = input.try_parse(non_negative_number) {
            if grow.is_none() && !factors_are_closed {
                grow = Some(number);
            } else if shrink.is_none() && !factors_are_closed {
                shrink = Some(number);
            } else if basis.is_none() && number == 0.0 {
                basis = Some(LengthPercentageAuto::LengthPercentage(zero_length()));
            } else {
                return Err(ValueError::Mismatch);
            }
        } else if basis.is_none() {
            basis = Some(size(tokens, input)?);
            factors_are_closed = grow.is_some();
        } else {
            return Err(ValueError::Mismatch);
        }
    }
    if grow.is_none() && basis.is_none() {
        return Err(ValueError::Mismatch);
    }
    Ok(longhands(
        grow.unwrap_or(1.0),
        shrink.unwrap_or(1.0),
        basis.unwrap_or(LengthPercentageAuto::LengthPercentage(zero_percent())),
    ))
}

fn longhands(grow: f32, shrink: f32, basis: LengthPercentageAuto) -> Vec<Declaration> {
    vec![
        Declaration::FlexGrow(grow),
        Declaration::FlexShrink(shrink),
        Declaration::FlexBasis(basis),
    ]
}

fn zero_length() -> LengthPercentage {
    LengthPercentage {
        pixels: Pixels(0.0),
        fraction: None,
        tokens: Cow::Borrowed(&[]),
    }
}

fn zero_percent() -> LengthPercentage {
    LengthPercentage {
        fraction: Some(0.0),
        ..zero_length()
    }
}
