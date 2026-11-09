use morok_style::{Color, LineStyle, ObjectFit, Shadow, Token};

use crate::geometry::corners::Corners;
use crate::geometry::sides::Sides;
use crate::style::declaration_source::DeclarationSource;
use crate::style::layout_style::MEDIUM_LINE_WIDTH;

pub(crate) const TRANSPARENT: Color = Color {
    red: 0,
    green: 0,
    blue: 0,
    alpha: 0.0,
};

/// The CSS initial `color`, which the root element inherits.
pub(crate) const INITIAL_COLOR: Color = Color {
    red: 0,
    green: 0,
    blue: 0,
    alpha: 1.0,
};

/// The part of a computed style that only painting reads; a change here moves no box.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PaintStyle {
    /// Inherited from the parent unless declared.
    pub color: Color,
    pub background_color: Color,
    pub(crate) background_image: Option<DeclarationSource>,
    pub border_color: Sides<Color>,
    pub border_radius: Corners<f32>,
    pub box_shadow: Option<Token<Shadow>>,
    /// The declared width; an outline without a style is not drawn, see [`Self::used_outline_width`].
    pub outline_width: f32,
    pub outline_style: LineStyle,
    pub outline_color: Color,
    pub outline_offset: f32,
    pub object_fit: ObjectFit,
}

impl PaintStyle {
    /// The CSS initial values for an element whose `color` is `color`: border and outline colors
    /// start as `currentColor`.
    pub(crate) const fn initial(color: Color) -> Self {
        Self {
            color,
            background_color: TRANSPARENT,
            background_image: None,
            border_color: Sides::uniform(color),
            border_radius: Corners {
                top_left: 0.0,
                top_right: 0.0,
                bottom_right: 0.0,
                bottom_left: 0.0,
            },
            box_shadow: None,
            outline_width: MEDIUM_LINE_WIDTH,
            outline_style: LineStyle::None,
            outline_color: color,
            outline_offset: 0.0,
            object_fit: ObjectFit::Fill,
        }
    }

    pub fn used_outline_width(&self) -> f32 {
        match self.outline_style {
            LineStyle::None => 0.0,
            LineStyle::Solid => self.outline_width,
        }
    }
}
