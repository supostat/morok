use morok_style::Theme;

use crate::geometry::rect::Rect;
use crate::geometry::sides::Sides;
use crate::style::computed_style::ComputedStyle;

/// A blurred shadow fades out over three standard deviations of its blur, and the standard
/// deviation is half the blur radius.
const BLUR_REACH_PER_RADIUS: f32 = 1.5;

/// Everything an element draws: its border box, the shadows around it and its outline.
pub(crate) fn visual_bounds(style: &ComputedStyle, border_box: &Rect, theme: &Theme) -> Rect {
    let mut bounds = *border_box;
    if let Some(shadow) = style.paint.box_shadow {
        for layer in theme.resolve(shadow).layers.iter().filter(|layer| !layer.inset) {
            let reach = layer.spread.0 + layer.blur.0 * BLUR_REACH_PER_RADIUS;
            let shadow_bounds = Rect::new(
                border_box.x + layer.offset_x.0,
                border_box.y + layer.offset_y.0,
                border_box.width,
                border_box.height,
            )
            .expanded(Sides::uniform(reach));
            bounds = bounds.union(&shadow_bounds);
        }
    }
    let outline_width = style.paint.used_outline_width();
    let outline_reach = style.paint.outline_offset + outline_width;
    if outline_width > 0.0 && outline_reach > 0.0 {
        bounds = bounds.union(&border_box.expanded(Sides::uniform(outline_reach)));
    }
    bounds
}
