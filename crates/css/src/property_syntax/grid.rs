use std::borrow::Cow;

use cssparser::{Parser, Token};
use morok_style::{Declaration, GridAutoFlow, GridLine};

use crate::problem::Problem;
use crate::property::Property;
use crate::property_syntax::{accepts_keyword, keyword, peek, track};
use crate::span::Span;
use crate::value_error::{ValueError, ValueResult};

const AUTO: &str = "auto";
const SPAN: &str = "span";
const DENSE: &str = "dense";
const FLOW_AXES: [(&str, GridAutoFlow); 2] = [("row", GridAutoFlow::Row), ("column", GridAutoFlow::Column)];

const TEMPLATE: &str = "`none` or a list of track sizes and `repeat()`";
const AUTOMATIC_TRACKS: &str = "one or more track sizes";
const LINE: &str = "`auto`, a line number or `span` with a count";
const PLACEMENT: &str = "a start line and, after `/`, an end line";

pub(crate) const PROPERTIES: &[Property] = &[
    Property {
        name: "grid-template-columns",
        expectation: TEMPLATE,
        parse: |context, input| {
            Ok(vec![Declaration::GridTemplateColumns(Cow::Owned(track::template(
                context.tokens,
                input,
            )?))])
        },
    },
    Property {
        name: "grid-template-rows",
        expectation: TEMPLATE,
        parse: |context, input| {
            Ok(vec![Declaration::GridTemplateRows(Cow::Owned(track::template(
                context.tokens,
                input,
            )?))])
        },
    },
    Property {
        name: "grid-auto-columns",
        expectation: AUTOMATIC_TRACKS,
        parse: |context, input| {
            Ok(vec![Declaration::GridAutoColumns(Cow::Owned(track::sizes(
                context.tokens,
                input,
            )?))])
        },
    },
    Property {
        name: "grid-auto-rows",
        expectation: AUTOMATIC_TRACKS,
        parse: |context, input| {
            Ok(vec![Declaration::GridAutoRows(Cow::Owned(track::sizes(
                context.tokens,
                input,
            )?))])
        },
    },
    Property {
        name: "grid-auto-flow",
        expectation: "`row` or `column`, with or without `dense`",
        parse: |_, input| Ok(vec![Declaration::GridAutoFlow(automatic_flow(input)?)]),
    },
    Property {
        name: "grid-column",
        expectation: PLACEMENT,
        parse: |_, input| {
            let (start, end) = placement(input)?;
            Ok(vec![
                Declaration::GridColumnStart(start),
                Declaration::GridColumnEnd(end),
            ])
        },
    },
    Property {
        name: "grid-row",
        expectation: PLACEMENT,
        parse: |_, input| {
            let (start, end) = placement(input)?;
            Ok(vec![Declaration::GridRowStart(start), Declaration::GridRowEnd(end)])
        },
    },
    Property {
        name: "grid-column-start",
        expectation: LINE,
        parse: |_, input| Ok(vec![Declaration::GridColumnStart(line(input)?)]),
    },
    Property {
        name: "grid-column-end",
        expectation: LINE,
        parse: |_, input| Ok(vec![Declaration::GridColumnEnd(line(input)?)]),
    },
    Property {
        name: "grid-row-start",
        expectation: LINE,
        parse: |_, input| Ok(vec![Declaration::GridRowStart(line(input)?)]),
    },
    Property {
        name: "grid-row-end",
        expectation: LINE,
        parse: |_, input| Ok(vec![Declaration::GridRowEnd(line(input)?)]),
    },
];

fn automatic_flow(input: &mut Parser<'_>) -> ValueResult<GridAutoFlow> {
    let is_dense_before_the_axis = accepts_keyword(input, DENSE);
    let axis = input.try_parse(|input| keyword(input, &FLOW_AXES)).ok();
    if axis.is_none() && !is_dense_before_the_axis {
        return Err(ValueError::Mismatch);
    }
    let is_dense = is_dense_before_the_axis || accepts_keyword(input, DENSE);
    Ok(match (axis.unwrap_or(GridAutoFlow::Row), is_dense) {
        (GridAutoFlow::Column | GridAutoFlow::ColumnDense, false) => GridAutoFlow::Column,
        (GridAutoFlow::Column | GridAutoFlow::ColumnDense, true) => GridAutoFlow::ColumnDense,
        (GridAutoFlow::Row | GridAutoFlow::RowDense, false) => GridAutoFlow::Row,
        (GridAutoFlow::Row | GridAutoFlow::RowDense, true) => GridAutoFlow::RowDense,
    })
}

/// A start line and an optional end line after `/`; an omitted end line is `auto`.
fn placement(input: &mut Parser<'_>) -> ValueResult<(GridLine, GridLine)> {
    let start = line(input)?;
    if input.is_exhausted() {
        return Ok((start, GridLine::Auto));
    }
    input.expect_delim('/').map_err(|_| ValueError::Mismatch)?;
    Ok((start, line(input)?))
}

fn line(input: &mut Parser<'_>) -> ValueResult<GridLine> {
    if accepts_keyword(input, AUTO) {
        return Ok(GridLine::Auto);
    }
    if accepts_keyword(input, SPAN) {
        let count = input.expect_integer().map_err(|_| ValueError::Mismatch)?;
        return match u16::try_from(count) {
            Ok(count) if count > 0 => Ok(GridLine::Span(count)),
            _ => Err(ValueError::Mismatch),
        };
    }
    input.skip_whitespace();
    let start = input.position();
    if let Some(Token::Ident(_)) = peek(input) {
        input.next().map_err(|_| ValueError::Mismatch)?;
        return Err(Problem {
            span: Span::between(start, input.position()),
            message: "named grid lines are not supported".to_owned(),
            hint: "place the item by line number or by `span`".to_owned(),
        }
        .into());
    }
    let number = input.expect_integer().map_err(|_| ValueError::Mismatch)?;
    match i16::try_from(number) {
        Ok(number) if number != 0 => Ok(GridLine::Line(number)),
        _ => Err(ValueError::Mismatch),
    }
}
