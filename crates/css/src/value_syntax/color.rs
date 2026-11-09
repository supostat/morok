use cssparser::{Parser, Token};
use morok_style::Color;

use super::number::unit_interval;
use super::{Parsed, rejected};

const OPAQUE: f32 = 1.0;

pub(crate) fn parse(input: &mut Parser<'_>) -> Parsed<Color> {
    match input.next()?.clone() {
        Token::Hash(digits) | Token::IDHash(digits) => hex(&digits),
        Token::Function(name) if name.eq_ignore_ascii_case("rgb") || name.eq_ignore_ascii_case("rgba") => {
            input.parse_nested_block(rgb_arguments)
        }
        _ => rejected(),
    }
}

fn hex(digits: &str) -> Parsed<Color> {
    let Some(nibbles) = digits
        .chars()
        .map(|digit| digit.to_digit(16))
        .collect::<Option<Vec<u32>>>()
    else {
        return rejected();
    };
    let channels: Vec<u8> = match nibbles.len() {
        3 | 4 => nibbles.iter().map(|nibble| (nibble * 17) as u8).collect(),
        6 | 8 => nibbles.chunks(2).map(|pair| (pair[0] * 16 + pair[1]) as u8).collect(),
        _ => return rejected(),
    };
    Ok(Color {
        red: channels[0],
        green: channels[1],
        blue: channels[2],
        alpha: channels.get(3).map_or(OPAQUE, |&alpha| f32::from(alpha) / 255.0),
    })
}

fn rgb_arguments(input: &mut Parser<'_>) -> Parsed<Color> {
    let red = channel(input)?;
    input.expect_comma()?;
    let green = channel(input)?;
    input.expect_comma()?;
    let blue = channel(input)?;
    let alpha = if input.try_parse(Parser::expect_comma).is_ok() {
        unit_interval(input)?
    } else {
        OPAQUE
    };
    Ok(Color {
        red,
        green,
        blue,
        alpha,
    })
}

fn channel(input: &mut Parser<'_>) -> Parsed<u8> {
    u8::try_from(input.expect_integer()?).or_else(|_| rejected())
}
