use std::borrow::Cow;

use cssparser::Parser;
use morok_style::{Pixels, Shadow, ShadowLayer};

use super::{Parsed, color, length, rejected};

const INSET: &str = "inset";

struct LayerGeometry {
    offset_x: Pixels,
    offset_y: Pixels,
    blur: Pixels,
    spread: Pixels,
}

pub(crate) fn parse(input: &mut Parser<'_>) -> Parsed<Shadow> {
    let layers = input.parse_comma_separated(layer)?;
    Ok(Shadow {
        layers: Cow::Owned(layers),
    })
}

/// One layer: `inset`, the color and the lengths come in any order, as in CSS.
fn layer(input: &mut Parser<'_>) -> Parsed<ShadowLayer> {
    let mut inset = false;
    let mut layer_color = None;
    let mut geometry = None;
    loop {
        if !inset && input.try_parse(|input| input.expect_ident_matching(INSET)).is_ok() {
            inset = true;
        } else if layer_color.is_none()
            && let Ok(parsed) = input.try_parse(color::parse)
        {
            layer_color = Some(parsed);
        } else if geometry.is_none()
            && let Ok(parsed) = input.try_parse(layer_geometry)
        {
            geometry = Some(parsed);
        } else {
            break;
        }
    }
    let (Some(color), Some(geometry)) = (layer_color, geometry) else {
        return rejected();
    };
    Ok(ShadowLayer {
        offset_x: geometry.offset_x,
        offset_y: geometry.offset_y,
        blur: geometry.blur,
        spread: geometry.spread,
        color,
        inset,
    })
}

fn layer_geometry(input: &mut Parser<'_>) -> Parsed<LayerGeometry> {
    let offset_x = length::parse(input)?;
    let offset_y = length::parse(input)?;
    let blur = input.try_parse(length::parse).unwrap_or(Pixels(0.0));
    let spread = input.try_parse(length::parse).unwrap_or(Pixels(0.0));
    if blur.0 < 0.0 {
        return rejected();
    }
    Ok(LayerGeometry {
        offset_x,
        offset_y,
        blur,
        spread,
    })
}
