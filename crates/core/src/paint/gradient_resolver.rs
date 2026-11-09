use std::f32::consts::SQRT_2;

use morok_style::{
    Color, ColorStop, ColorValue, Corner, Gradient, GradientDirection, LengthPercentage, LinearGradient, RadialExtent,
    RadialGradient, RadialShape, Theme,
};

use crate::geometry::point::Point;
use crate::geometry::rect::Rect;
use crate::paint::gradient_fill::GradientFill;
use crate::paint::gradient_shape::GradientShape;
use crate::paint::gradient_stop::GradientStop;
use crate::paint::item_range::ItemRange;
use crate::style::paint_style::TRANSPARENT;
use crate::style::resolved_length::ResolvedLength;

/// Resolves a gradient against the padding box of its element, as CSS Images defines it.
#[derive(Clone, Copy, Debug)]
pub(crate) struct GradientResolver<'a> {
    pub(crate) theme: &'a Theme,
    /// The `color` of the element, for stops that say `currentColor`.
    pub(crate) current_color: Color,
}

impl GradientResolver<'_> {
    /// Appends the stops of the gradient to `stops` and returns the gradient that refers to them.
    pub(crate) fn resolve(
        self,
        gradient: &Gradient,
        padding_box: &Rect,
        stops: &mut Vec<GradientStop>,
    ) -> GradientFill {
        match gradient {
            Gradient::Linear(linear) => self.linear(linear, padding_box, stops),
            Gradient::Radial(radial) => self.radial(radial, padding_box, stops),
        }
    }

    fn linear(self, gradient: &LinearGradient, padding_box: &Rect, stops: &mut Vec<GradientStop>) -> GradientFill {
        let angle = match gradient.direction {
            GradientDirection::Angle(angle) => angle,
            GradientDirection::ToCorner(corner) => angle_towards_corner(corner, padding_box),
        };
        let radians = angle.to_radians();
        let line_length = (padding_box.width * radians.sin()).abs() + (padding_box.height * radians.cos()).abs();
        GradientFill {
            shape: GradientShape::Linear { angle },
            stops: self.push_stops(&gradient.stops, line_length, stops),
        }
    }

    fn radial(self, gradient: &RadialGradient, padding_box: &Rect, stops: &mut Vec<GradientStop>) -> GradientFill {
        let center_x = ResolvedLength::of(&gradient.center_x, self.theme).against(padding_box.width);
        let center_y = ResolvedLength::of(&gradient.center_y, self.theme).against(padding_box.height);
        let (horizontal_radius, vertical_radius) = radial_radii(
            gradient.shape,
            gradient.extent,
            Point {
                x: center_x,
                y: center_y,
            },
            padding_box,
        );
        GradientFill {
            shape: GradientShape::Radial {
                center: Point {
                    x: padding_box.x + center_x,
                    y: padding_box.y + center_y,
                },
                horizontal_radius,
                vertical_radius,
            },
            stops: self.push_stops(&gradient.stops, horizontal_radius, stops),
        }
    }

    /// `line_length` is the length a stop position in pixels is a part of.
    fn push_stops(self, declared: &[ColorStop], line_length: f32, stops: &mut Vec<GradientStop>) -> ItemRange {
        let start = stops.len();
        stops.extend(declared.iter().map(|stop| {
            GradientStop {
                offset: stop
                    .position
                    .as_ref()
                    .map_or(0.0, |position| self.stop_offset(position, line_length)),
                color: self.color(stop.color),
            }
        }));
        place_stops(declared, &mut stops[start..]);
        ItemRange {
            start: start as u32,
            end: stops.len() as u32,
        }
    }

    fn stop_offset(self, position: &LengthPercentage, line_length: f32) -> f32 {
        let position = ResolvedLength::of(position, self.theme);
        let pixels_share = if line_length > 0.0 {
            position.pixels / line_length
        } else {
            0.0
        };
        pixels_share + position.fraction.unwrap_or(0.0)
    }

    fn color(self, color: ColorValue) -> Color {
        match color {
            ColorValue::Token(token) => *self.theme.resolve(token),
            ColorValue::Transparent => TRANSPARENT,
            ColorValue::CurrentColor => self.current_color,
        }
    }
}

/// The angle of a gradient towards a corner: its line is perpendicular to the diagonal between
/// the two neighbouring corners, so the middle color runs exactly through them.
fn angle_towards_corner(corner: Corner, padding_box: &Rect) -> f32 {
    let from_vertical = padding_box.height.atan2(padding_box.width).to_degrees();
    match corner {
        Corner::TopRight => from_vertical,
        Corner::BottomRight => 180.0 - from_vertical,
        Corner::BottomLeft => 180.0 + from_vertical,
        Corner::TopLeft => 360.0 - from_vertical,
    }
}

/// `center` is relative to the padding box.
fn radial_radii(shape: RadialShape, extent: RadialExtent, center: Point, padding_box: &Rect) -> (f32, f32) {
    let to_left = center.x.abs();
    let to_right = (padding_box.width - center.x).abs();
    let to_top = center.y.abs();
    let to_bottom = (padding_box.height - center.y).abs();
    let (horizontal, vertical) = match extent {
        RadialExtent::ClosestSide | RadialExtent::ClosestCorner => (to_left.min(to_right), to_top.min(to_bottom)),
        RadialExtent::FarthestSide | RadialExtent::FarthestCorner => (to_left.max(to_right), to_top.max(to_bottom)),
    };
    match (shape, extent) {
        (RadialShape::Circle, RadialExtent::ClosestSide) => {
            let radius = horizontal.min(vertical);
            (radius, radius)
        }
        (RadialShape::Circle, RadialExtent::FarthestSide) => {
            let radius = horizontal.max(vertical);
            (radius, radius)
        }
        (RadialShape::Circle, RadialExtent::ClosestCorner | RadialExtent::FarthestCorner) => {
            let radius = horizontal.hypot(vertical);
            (radius, radius)
        }
        (RadialShape::Ellipse, RadialExtent::ClosestSide | RadialExtent::FarthestSide) => (horizontal, vertical),
        (RadialShape::Ellipse, RadialExtent::ClosestCorner | RadialExtent::FarthestCorner) => {
            (horizontal * SQRT_2, vertical * SQRT_2)
        }
    }
}

/// Gives every stop its final offset, as CSS fixes stops up: a first stop without a position
/// starts at 0 and a last one ends at 1, no stop lies before the one it follows, and stops
/// without a position are spread evenly between their positioned neighbours.
fn place_stops(declared: &[ColorStop], stops: &mut [GradientStop]) {
    let Some(last) = stops.len().checked_sub(1) else {
        return;
    };
    if declared[0].position.is_none() {
        stops[0].offset = 0.0;
    }
    if declared[last].position.is_none() {
        stops[last].offset = 1.0;
    }
    let is_positioned = |stop: usize| stop == 0 || stop == last || declared[stop].position.is_some();
    let mut positioned_before = 0;
    for stop in 1..=last {
        if !is_positioned(stop) {
            continue;
        }
        let from = stops[positioned_before].offset;
        stops[stop].offset = stops[stop].offset.max(from);
        let intervals = stop - positioned_before;
        let interval = (stops[stop].offset - from) / intervals as f32;
        for unpositioned in 1..intervals {
            stops[positioned_before + unpositioned].offset = from + interval * unpositioned as f32;
        }
        positioned_before = stop;
    }
}

#[cfg(test)]
mod tests {
    use std::borrow::Cow;

    use morok_style::{
        Color, ColorStop, ColorValue, Corner, Gradient, GradientDirection, LengthPercentage, LinearGradient, Pixels,
        RadialExtent, RadialGradient, RadialShape, Theme,
    };

    use super::GradientResolver;
    use crate::geometry::point::Point;
    use crate::geometry::rect::Rect;
    use crate::paint::gradient_shape::GradientShape;
    use crate::paint::gradient_stop::GradientStop;
    use crate::style::paint_style::TRANSPARENT;

    const CURRENT_COLOR: Color = Color {
        red: 10,
        green: 20,
        blue: 30,
        alpha: 1.0,
    };

    fn empty_theme() -> Theme {
        Theme {
            colors: Cow::Borrowed(&[]),
            lengths: Cow::Borrowed(&[]),
            font_families: Cow::Borrowed(&[]),
            leadings: Cow::Borrowed(&[]),
            font_weights: Cow::Borrowed(&[]),
            shadows: Cow::Borrowed(&[]),
            durations: Cow::Borrowed(&[]),
            easings: Cow::Borrowed(&[]),
        }
    }

    fn length(pixels: f32, fraction: Option<f32>) -> LengthPercentage {
        LengthPercentage {
            pixels: Pixels(pixels),
            fraction,
            tokens: Cow::Borrowed(&[]),
        }
    }

    fn stop(position: Option<LengthPercentage>) -> ColorStop {
        ColorStop {
            color: ColorValue::CurrentColor,
            position,
        }
    }

    fn linear(direction: GradientDirection, stops: Vec<ColorStop>) -> Gradient {
        Gradient::Linear(LinearGradient {
            direction,
            stops: Cow::Owned(stops),
        })
    }

    fn radial(shape: RadialShape, extent: RadialExtent, center: (LengthPercentage, LengthPercentage)) -> Gradient {
        Gradient::Radial(RadialGradient {
            shape,
            extent,
            center_x: center.0,
            center_y: center.1,
            stops: Cow::Owned(vec![stop(None), stop(None)]),
        })
    }

    fn resolved(gradient: &Gradient, padding_box: Rect) -> (GradientShape, Vec<GradientStop>) {
        let theme = empty_theme();
        let resolver = GradientResolver {
            theme: &theme,
            current_color: CURRENT_COLOR,
        };
        let mut stops = Vec::new();
        let fill = resolver.resolve(gradient, &padding_box, &mut stops);
        (fill.shape, stops[fill.stops.as_range()].to_vec())
    }

    fn offsets(stops: &[GradientStop]) -> Vec<f32> {
        stops.iter().map(|stop| stop.offset).collect()
    }

    fn assert_close(actual: f32, expected: f32) {
        assert!((actual - expected).abs() < 1e-3, "{actual} is not {expected}");
    }

    #[test]
    fn stops_without_positions_are_spread_evenly_from_zero_to_one() {
        let gradient = linear(
            GradientDirection::Angle(180.0),
            vec![stop(None), stop(None), stop(None)],
        );

        let (_, stops) = resolved(&gradient, Rect::new(0.0, 0.0, 100.0, 50.0));

        assert_eq!(offsets(&stops), [0.0, 0.5, 1.0]);
    }

    #[test]
    fn stops_between_positioned_neighbours_are_spread_between_them() {
        let gradient = linear(
            GradientDirection::Angle(90.0),
            vec![
                stop(Some(length(0.0, Some(0.2)))),
                stop(None),
                stop(None),
                stop(Some(length(0.0, Some(0.8)))),
                stop(None),
            ],
        );

        let (_, stops) = resolved(&gradient, Rect::new(0.0, 0.0, 100.0, 50.0));

        let offsets = offsets(&stops);
        assert_close(offsets[0], 0.2);
        assert_close(offsets[1], 0.4);
        assert_close(offsets[2], 0.6);
        assert_close(offsets[3], 0.8);
        assert_close(offsets[4], 1.0);
    }

    #[test]
    fn a_stop_placed_before_its_predecessor_moves_up_to_it() {
        let gradient = linear(
            GradientDirection::Angle(90.0),
            vec![
                stop(Some(length(0.0, Some(0.6)))),
                stop(Some(length(0.0, Some(0.3)))),
                stop(None),
            ],
        );

        let (_, stops) = resolved(&gradient, Rect::new(0.0, 0.0, 100.0, 50.0));

        assert_eq!(offsets(&stops), [0.6, 0.6, 1.0]);
    }

    #[test]
    fn a_stop_in_pixels_is_a_share_of_the_gradient_line() {
        let horizontal = linear(
            GradientDirection::Angle(90.0),
            vec![stop(None), stop(Some(length(50.0, None)))],
        );
        let vertical = linear(
            GradientDirection::Angle(0.0),
            vec![stop(None), stop(Some(length(50.0, None)))],
        );
        let padding_box = Rect::new(0.0, 0.0, 200.0, 100.0);

        assert_eq!(offsets(&resolved(&horizontal, padding_box).1), [0.0, 0.25]);
        assert_eq!(offsets(&resolved(&vertical, padding_box).1), [0.0, 0.5]);
    }

    #[test]
    fn a_stop_takes_its_color_from_the_keyword_it_names() {
        let gradient = linear(
            GradientDirection::Angle(0.0),
            vec![
                stop(None),
                ColorStop {
                    color: ColorValue::Transparent,
                    position: None,
                },
            ],
        );

        let (_, stops) = resolved(&gradient, Rect::new(0.0, 0.0, 10.0, 10.0));

        assert_eq!(stops[0].color, CURRENT_COLOR);
        assert_eq!(stops[1].color, TRANSPARENT);
    }

    #[test]
    fn a_gradient_towards_a_corner_runs_across_the_diagonal_of_the_other_two() {
        let padding_box = Rect::new(0.0, 0.0, 200.0, 100.0);
        let towards = |corner| {
            let gradient = linear(GradientDirection::ToCorner(corner), vec![stop(None), stop(None)]);
            match resolved(&gradient, padding_box).0 {
                GradientShape::Linear { angle } => angle,
                GradientShape::Radial { .. } => panic!("the gradient is linear"),
            }
        };
        let from_vertical = 100.0_f32.atan2(200.0).to_degrees();

        assert_close(towards(Corner::TopRight), from_vertical);
        assert_close(towards(Corner::BottomRight), 180.0 - from_vertical);
        assert_close(towards(Corner::BottomLeft), 180.0 + from_vertical);
        assert_close(towards(Corner::TopLeft), 360.0 - from_vertical);
    }

    #[test]
    fn a_gradient_towards_a_corner_of_a_square_is_diagonal() {
        let gradient = linear(
            GradientDirection::ToCorner(Corner::BottomRight),
            vec![stop(None), stop(None)],
        );

        let (shape, _) = resolved(&gradient, Rect::new(0.0, 0.0, 80.0, 80.0));

        assert_eq!(shape, GradientShape::Linear { angle: 135.0 });
    }

    #[test]
    fn a_circle_reaches_the_side_or_the_corner_its_extent_names() {
        let padding_box = Rect::new(10.0, 20.0, 200.0, 100.0);
        let center = || (length(0.0, Some(0.25)), length(40.0, None));
        let radius = |extent| match resolved(&radial(RadialShape::Circle, extent, center()), padding_box).0 {
            GradientShape::Radial {
                center,
                horizontal_radius,
                vertical_radius,
            } => {
                assert_eq!(center, Point { x: 60.0, y: 60.0 });
                assert_close(horizontal_radius, vertical_radius);
                horizontal_radius
            }
            GradientShape::Linear { .. } => panic!("the gradient is radial"),
        };

        assert_close(radius(RadialExtent::ClosestSide), 40.0);
        assert_close(radius(RadialExtent::FarthestSide), 150.0);
        assert_close(radius(RadialExtent::ClosestCorner), 50.0_f32.hypot(40.0));
        assert_close(radius(RadialExtent::FarthestCorner), 150.0_f32.hypot(60.0));
    }

    #[test]
    fn an_ellipse_keeps_the_proportions_of_its_side_distances() {
        let padding_box = Rect::new(0.0, 0.0, 200.0, 100.0);
        let center = || (length(0.0, Some(0.25)), length(40.0, None));
        let radii = |extent| match resolved(&radial(RadialShape::Ellipse, extent, center()), padding_box).0 {
            GradientShape::Radial {
                horizontal_radius,
                vertical_radius,
                ..
            } => (horizontal_radius, vertical_radius),
            GradientShape::Linear { .. } => panic!("the gradient is radial"),
        };

        assert_eq!(radii(RadialExtent::ClosestSide), (50.0, 40.0));
        assert_eq!(radii(RadialExtent::FarthestSide), (150.0, 60.0));
        let (horizontal, vertical) = radii(RadialExtent::FarthestCorner);
        assert_close(horizontal, 150.0 * std::f32::consts::SQRT_2);
        assert_close(vertical, 60.0 * std::f32::consts::SQRT_2);
    }

    #[test]
    fn a_radial_stop_in_pixels_is_a_share_of_the_horizontal_radius() {
        let gradient = Gradient::Radial(RadialGradient {
            shape: RadialShape::Circle,
            extent: RadialExtent::ClosestSide,
            center_x: length(0.0, Some(0.5)),
            center_y: length(0.0, Some(0.5)),
            stops: Cow::Owned(vec![stop(None), stop(Some(length(12.0, None)))]),
        });

        let (_, stops) = resolved(&gradient, Rect::new(0.0, 0.0, 200.0, 96.0));

        assert_eq!(offsets(&stops), [0.0, 0.25]);
    }
}
