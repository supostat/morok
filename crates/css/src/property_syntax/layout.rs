use cssparser::Parser;
use morok_style::{Declaration, Display, LengthPercentage, LengthPercentageAuto, Overflow, Position, Side};

use crate::compiled_tokens::CompiledTokens;
use crate::property::Property;
use crate::property_syntax::{four_values, keyword, length};
use crate::value_error::ValueResult;

const DISPLAYS: [(&str, Display); 4] = [
    ("block", Display::Block),
    ("flex", Display::Flex),
    ("grid", Display::Grid),
    ("none", Display::None),
];
const POSITIONS: [(&str, Position); 2] = [("relative", Position::Relative), ("absolute", Position::Absolute)];
const OVERFLOWS: [(&str, Overflow); 2] = [("visible", Overflow::Visible), ("hidden", Overflow::Hidden)];

const AUTO: &str = "auto";
const NONE: &str = "none";

const OVERFLOW: &str = "`visible` or `hidden`";
const SIZE: &str = "a length, a percentage, `calc()` or `auto`";
const MAXIMUM_SIZE: &str = "a length, a percentage, `calc()` or `none`";
const FOUR_SIZES: &str = "one to four lengths, percentages or `auto`";
const PADDING: &str = "a length, a percentage or `calc()`, not negative";
const FOUR_PADDINGS: &str = "one to four lengths or percentages, not negative";
const GAP: &str = "a length, a percentage or `calc()`, not negative";

pub(crate) const PROPERTIES: &[Property] = &[
    Property {
        name: "display",
        expectation: "`block`, `flex`, `grid` or `none`",
        parse: |_, input| Ok(vec![Declaration::Display(keyword(input, &DISPLAYS)?)]),
    },
    Property {
        name: "position",
        expectation: "`relative` or `absolute`",
        parse: |_, input| Ok(vec![Declaration::Position(keyword(input, &POSITIONS)?)]),
    },
    Property {
        name: "overflow",
        expectation: "one or two of `visible` and `hidden`",
        parse: |_, input| {
            let horizontal = keyword(input, &OVERFLOWS)?;
            let vertical = if input.is_exhausted() {
                horizontal
            } else {
                keyword(input, &OVERFLOWS)?
            };
            Ok(vec![
                Declaration::OverflowX(horizontal),
                Declaration::OverflowY(vertical),
            ])
        },
    },
    Property {
        name: "overflow-x",
        expectation: OVERFLOW,
        parse: |_, input| Ok(vec![Declaration::OverflowX(keyword(input, &OVERFLOWS)?)]),
    },
    Property {
        name: "overflow-y",
        expectation: OVERFLOW,
        parse: |_, input| Ok(vec![Declaration::OverflowY(keyword(input, &OVERFLOWS)?)]),
    },
    Property {
        name: "width",
        expectation: SIZE,
        parse: |context, input| Ok(vec![Declaration::Width(size(context.tokens, input)?)]),
    },
    Property {
        name: "height",
        expectation: SIZE,
        parse: |context, input| Ok(vec![Declaration::Height(size(context.tokens, input)?)]),
    },
    Property {
        name: "min-width",
        expectation: SIZE,
        parse: |context, input| Ok(vec![Declaration::MinWidth(size(context.tokens, input)?)]),
    },
    Property {
        name: "min-height",
        expectation: SIZE,
        parse: |context, input| Ok(vec![Declaration::MinHeight(size(context.tokens, input)?)]),
    },
    Property {
        name: "max-width",
        expectation: MAXIMUM_SIZE,
        parse: |context, input| Ok(vec![Declaration::MaxWidth(maximum_size(context.tokens, input)?)]),
    },
    Property {
        name: "max-height",
        expectation: MAXIMUM_SIZE,
        parse: |context, input| Ok(vec![Declaration::MaxHeight(maximum_size(context.tokens, input)?)]),
    },
    Property {
        name: "margin",
        expectation: FOUR_SIZES,
        parse: |context, input| {
            let margins = four_values(input, |value| size_or_auto(context.tokens, value))?;
            Ok(on_every_side(margins, Declaration::Margin))
        },
    },
    Property {
        name: "margin-top",
        expectation: SIZE,
        parse: |context, input| {
            Ok(vec![Declaration::Margin(
                Side::Top,
                size_or_auto(context.tokens, input)?,
            )])
        },
    },
    Property {
        name: "margin-right",
        expectation: SIZE,
        parse: |context, input| {
            Ok(vec![Declaration::Margin(
                Side::Right,
                size_or_auto(context.tokens, input)?,
            )])
        },
    },
    Property {
        name: "margin-bottom",
        expectation: SIZE,
        parse: |context, input| {
            Ok(vec![Declaration::Margin(
                Side::Bottom,
                size_or_auto(context.tokens, input)?,
            )])
        },
    },
    Property {
        name: "margin-left",
        expectation: SIZE,
        parse: |context, input| {
            Ok(vec![Declaration::Margin(
                Side::Left,
                size_or_auto(context.tokens, input)?,
            )])
        },
    },
    Property {
        name: "padding",
        expectation: FOUR_PADDINGS,
        parse: |context, input| {
            let paddings = four_values(input, |value| padding(context.tokens, value))?;
            Ok(on_every_side(paddings, Declaration::Padding))
        },
    },
    Property {
        name: "padding-top",
        expectation: PADDING,
        parse: |context, input| Ok(vec![Declaration::Padding(Side::Top, padding(context.tokens, input)?)]),
    },
    Property {
        name: "padding-right",
        expectation: PADDING,
        parse: |context, input| Ok(vec![Declaration::Padding(Side::Right, padding(context.tokens, input)?)]),
    },
    Property {
        name: "padding-bottom",
        expectation: PADDING,
        parse: |context, input| {
            Ok(vec![Declaration::Padding(
                Side::Bottom,
                padding(context.tokens, input)?,
            )])
        },
    },
    Property {
        name: "padding-left",
        expectation: PADDING,
        parse: |context, input| Ok(vec![Declaration::Padding(Side::Left, padding(context.tokens, input)?)]),
    },
    Property {
        name: "inset",
        expectation: FOUR_SIZES,
        parse: |context, input| {
            let insets = four_values(input, |value| size_or_auto(context.tokens, value))?;
            Ok(on_every_side(insets, Declaration::Inset))
        },
    },
    Property {
        name: "top",
        expectation: SIZE,
        parse: |context, input| {
            Ok(vec![Declaration::Inset(
                Side::Top,
                size_or_auto(context.tokens, input)?,
            )])
        },
    },
    Property {
        name: "right",
        expectation: SIZE,
        parse: |context, input| {
            Ok(vec![Declaration::Inset(
                Side::Right,
                size_or_auto(context.tokens, input)?,
            )])
        },
    },
    Property {
        name: "bottom",
        expectation: SIZE,
        parse: |context, input| {
            Ok(vec![Declaration::Inset(
                Side::Bottom,
                size_or_auto(context.tokens, input)?,
            )])
        },
    },
    Property {
        name: "left",
        expectation: SIZE,
        parse: |context, input| {
            Ok(vec![Declaration::Inset(
                Side::Left,
                size_or_auto(context.tokens, input)?,
            )])
        },
    },
    Property {
        name: "gap",
        expectation: "one or two lengths or percentages, not negative",
        parse: |context, input| {
            let between_rows = padding(context.tokens, input)?;
            let between_columns = if input.is_exhausted() {
                between_rows.clone()
            } else {
                padding(context.tokens, input)?
            };
            Ok(vec![
                Declaration::RowGap(between_rows),
                Declaration::ColumnGap(between_columns),
            ])
        },
    },
    Property {
        name: "row-gap",
        expectation: GAP,
        parse: |context, input| Ok(vec![Declaration::RowGap(padding(context.tokens, input)?)]),
    },
    Property {
        name: "column-gap",
        expectation: GAP,
        parse: |context, input| Ok(vec![Declaration::ColumnGap(padding(context.tokens, input)?)]),
    },
];

/// A width, height or flex basis: never negative.
pub(crate) fn size(tokens: &CompiledTokens, input: &mut Parser<'_>) -> ValueResult<LengthPercentageAuto> {
    length::non_negative_length_percentage_or(AUTO, tokens, input)
}

fn maximum_size(tokens: &CompiledTokens, input: &mut Parser<'_>) -> ValueResult<LengthPercentageAuto> {
    length::non_negative_length_percentage_or(NONE, tokens, input)
}

/// A margin or an inset: may be negative.
fn size_or_auto(tokens: &CompiledTokens, input: &mut Parser<'_>) -> ValueResult<LengthPercentageAuto> {
    length::length_percentage_or(AUTO, tokens, input)
}

/// A padding or a gap: never negative.
fn padding(tokens: &CompiledTokens, input: &mut Parser<'_>) -> ValueResult<LengthPercentage> {
    length::non_negative_length_percentage(tokens, input)
}

pub(crate) fn on_every_side<T>(values: [T; 4], declaration: fn(Side, T) -> Declaration) -> Vec<Declaration> {
    Side::ALL
        .into_iter()
        .zip(values)
        .map(|(side, value)| declaration(side, value))
        .collect()
}
