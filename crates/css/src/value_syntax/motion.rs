use cssparser::{Parser, Token};
use morok_style::{Easing, Milliseconds, StepPosition};

use super::number::{finite, unit_interval};
use super::{Parsed, rejected};

const MILLISECONDS_PER_SECOND: f32 = 1000.0;

pub(crate) fn duration(input: &mut Parser<'_>) -> Parsed<Milliseconds> {
    let milliseconds = match input.next()?.clone() {
        Token::Dimension { value, unit, .. } if unit.eq_ignore_ascii_case("ms") => value,
        Token::Dimension { value, unit, .. } if unit.eq_ignore_ascii_case("s") => value * MILLISECONDS_PER_SECOND,
        _ => return rejected(),
    };
    if milliseconds >= 0.0 && milliseconds.is_finite() {
        Ok(Milliseconds(milliseconds))
    } else {
        rejected()
    }
}

pub(crate) fn easing(input: &mut Parser<'_>) -> Parsed<Easing> {
    match input.next()?.clone() {
        Token::Ident(name) if name.eq_ignore_ascii_case("linear") => Ok(Easing::Linear),
        Token::Function(name) if name.eq_ignore_ascii_case("cubic-bezier") => {
            input.parse_nested_block(cubic_bezier_arguments)
        }
        Token::Function(name) if name.eq_ignore_ascii_case("steps") => input.parse_nested_block(steps_arguments),
        _ => rejected(),
    }
}

fn cubic_bezier_arguments(input: &mut Parser<'_>) -> Parsed<Easing> {
    let x1 = unit_interval(input)?;
    input.expect_comma()?;
    let y1 = finite(input)?;
    input.expect_comma()?;
    let x2 = unit_interval(input)?;
    input.expect_comma()?;
    let y2 = finite(input)?;
    Ok(Easing::CubicBezier { x1, y1, x2, y2 })
}

fn steps_arguments(input: &mut Parser<'_>) -> Parsed<Easing> {
    let count = input.expect_integer()?;
    let position = if input.try_parse(Parser::expect_comma).is_ok() {
        step_position(input)?
    } else {
        StepPosition::JumpEnd
    };
    let fewest_steps = if position == StepPosition::JumpNone { 2 } else { 1 };
    if count < fewest_steps {
        return rejected();
    }
    Ok(Easing::Steps {
        count: count as u32,
        position,
    })
}

fn step_position(input: &mut Parser<'_>) -> Parsed<StepPosition> {
    match input.expect_ident_cloned()?.to_ascii_lowercase().as_str() {
        "jump-start" | "start" => Ok(StepPosition::JumpStart),
        "jump-end" | "end" => Ok(StepPosition::JumpEnd),
        "jump-none" => Ok(StepPosition::JumpNone),
        "jump-both" => Ok(StepPosition::JumpBoth),
        _ => rejected(),
    }
}
