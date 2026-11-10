use std::fmt::Write;

use morok_style::{Declaration, LengthPercentage, LengthPercentageAuto, Position, Side};

/// The declarations set on an element directly, written as the value of its `style` attribute.
/// The stand writes the geometry a component places by its state: positions, sizes, insets and
/// paddings in pixels and percentages.
///
/// # Panics
///
/// When a declaration is one the stand writes no CSS for, or refers to a token.
pub(crate) fn style_attribute(declarations: &[Declaration]) -> String {
    let mut style = String::new();
    for declaration in declarations {
        let (property, value) = match declaration {
            Declaration::Width(value) => ("width", css_length(value)),
            Declaration::Height(value) => ("height", css_length(value)),
            Declaration::Inset(side, value) => (inset_property(*side), css_length(value)),
            Declaration::Padding(side, value) => (
                padding_property(*side),
                css_length(&LengthPercentageAuto::LengthPercentage(value.clone())),
            ),
            Declaration::Position(position) => ("position", String::from(position_keyword(*position))),
            unwritten => panic!("the stand writes no inline CSS for {unwritten:?}"),
        };
        if !style.is_empty() {
            style.push(' ');
        }
        let _ = write!(style, "{property}: {value};");
    }
    style
}

const fn inset_property(side: Side) -> &'static str {
    match side {
        Side::Top => "top",
        Side::Right => "right",
        Side::Bottom => "bottom",
        Side::Left => "left",
    }
}

const fn padding_property(side: Side) -> &'static str {
    match side {
        Side::Top => "padding-top",
        Side::Right => "padding-right",
        Side::Bottom => "padding-bottom",
        Side::Left => "padding-left",
    }
}

const fn position_keyword(position: Position) -> &'static str {
    match position {
        Position::Relative => "relative",
        Position::Absolute => "absolute",
        Position::Sticky => "sticky",
    }
}

fn css_length(value: &LengthPercentageAuto) -> String {
    let LengthPercentageAuto::LengthPercentage(LengthPercentage {
        pixels,
        fraction,
        tokens,
    }) = value
    else {
        return String::from("auto");
    };
    assert!(
        tokens.is_empty(),
        "the stand writes no inline CSS that refers to a token"
    );
    let pixels = f64::from(pixels.0);
    match fraction {
        None => format!("{pixels}px"),
        Some(fraction) => {
            let percent = f64::from(*fraction) * 100.0;
            if pixels == 0.0 {
                format!("{percent}%")
            } else if pixels < 0.0 {
                format!("calc({percent}% - {}px)", -pixels)
            } else {
                format!("calc({percent}% + {pixels}px)")
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::borrow::Cow;

    use morok_style::{Declaration, LengthPercentage, LengthPercentageAuto, Pixels, Side};

    use super::style_attribute;

    fn length(pixels: f32, fraction: Option<f32>) -> LengthPercentageAuto {
        LengthPercentageAuto::LengthPercentage(LengthPercentage {
            pixels: Pixels(pixels),
            fraction,
            tokens: Cow::Borrowed(&[]),
        })
    }

    #[test]
    fn sizes_and_insets_are_written_in_pixels_percentages_and_their_sums() {
        let style = style_attribute(&[
            Declaration::Width(length(0.0, Some(0.25))),
            Declaration::Inset(Side::Left, length(-9.0, Some(0.5))),
            Declaration::Height(length(12.0, None)),
            Declaration::Inset(Side::Top, LengthPercentageAuto::Auto),
        ]);

        assert_eq!(style, "width: 25%; left: calc(50% - 9px); height: 12px; top: auto;");
    }
}
