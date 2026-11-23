use crate::geometry::point::Point;
use crate::geometry::rect::Rect;

/// A 2D affine transform in CSS pixels, as CSS `matrix(a, b, c, d, tx, ty)` writes it:
/// a point `(x, y)` maps to `(a·x + c·y + tx, b·x + d·y + ty)`. The translation is in double
/// precision, so that the scroll of a long content and the displacement of what lies far down
/// it cancel out to the pixel.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Transform {
    pub a: f32,
    pub b: f32,
    pub c: f32,
    pub d: f32,
    pub tx: f64,
    pub ty: f64,
}

impl Transform {
    pub const IDENTITY: Self = Self {
        a: 1.0,
        b: 0.0,
        c: 0.0,
        d: 1.0,
        tx: 0.0,
        ty: 0.0,
    };

    pub const fn translation(x: f64, y: f64) -> Self {
        Self {
            tx: x,
            ty: y,
            ..Self::IDENTITY
        }
    }

    pub const fn scaling(x: f32, y: f32) -> Self {
        Self {
            a: x,
            d: y,
            ..Self::IDENTITY
        }
    }

    /// A rotation clockwise on a surface whose `y` grows downwards.
    pub fn rotation(degrees: f32) -> Self {
        let (sine, cosine) = degrees.to_radians().sin_cos();
        Self {
            a: cosine,
            b: sine,
            c: -sine,
            d: cosine,
            ..Self::IDENTITY
        }
    }

    pub fn is_identity(&self) -> bool {
        *self == Self::IDENTITY
    }

    /// The transform that applies `self` first and `next` after it.
    #[must_use]
    pub fn then(&self, next: &Self) -> Self {
        Self {
            a: next.a * self.a + next.c * self.b,
            b: next.b * self.a + next.d * self.b,
            c: next.a * self.c + next.c * self.d,
            d: next.b * self.c + next.d * self.d,
            tx: f64::from(next.a) * self.tx + f64::from(next.c) * self.ty + next.tx,
            ty: f64::from(next.b) * self.tx + f64::from(next.d) * self.ty + next.ty,
        }
    }

    pub fn apply(&self, point: Point) -> Point {
        Point {
            x: surface_coordinate(f64::from(self.a * point.x + self.c * point.y) + self.tx),
            y: surface_coordinate(f64::from(self.b * point.x + self.d * point.y) + self.ty),
        }
    }

    /// The axis-aligned rectangle around the transformed corners of `rect`.
    pub fn bounds_of(&self, rect: &Rect) -> Rect {
        let corners = [
            self.apply(Point { x: rect.x, y: rect.y }),
            self.apply(Point {
                x: rect.right(),
                y: rect.y,
            }),
            self.apply(Point {
                x: rect.x,
                y: rect.bottom(),
            }),
            self.apply(Point {
                x: rect.right(),
                y: rect.bottom(),
            }),
        ];
        let left = corners.iter().map(|corner| corner.x).fold(f32::INFINITY, f32::min);
        let top = corners.iter().map(|corner| corner.y).fold(f32::INFINITY, f32::min);
        let right = corners.iter().map(|corner| corner.x).fold(f32::NEG_INFINITY, f32::max);
        let bottom = corners.iter().map(|corner| corner.y).fold(f32::NEG_INFINITY, f32::max);
        Rect::new(left, top, right - left, bottom - top)
    }

    pub fn determinant(&self) -> f32 {
        self.a * self.d - self.b * self.c
    }

    /// The transform that undoes this one; none when it collapses the plane onto a line.
    pub fn inverse(&self) -> Option<Self> {
        let determinant = self.determinant();
        if determinant == 0.0 || !determinant.is_finite() {
            return None;
        }
        let a = self.d / determinant;
        let b = -self.b / determinant;
        let c = -self.c / determinant;
        let d = self.a / determinant;
        Some(Self {
            a,
            b,
            c,
            d,
            tx: -(f64::from(a) * self.tx + f64::from(c) * self.ty),
            ty: -(f64::from(b) * self.tx + f64::from(d) * self.ty),
        })
    }
}

/// A coordinate in single precision, as boxes and points are.
#[expect(
    clippy::cast_possible_truncation,
    reason = "boxes and points are single precision; only the translation is double"
)]
fn surface_coordinate(coordinate: f64) -> f32 {
    coordinate as f32
}

#[cfg(test)]
mod tests {
    use super::Transform;
    use crate::geometry::point::Point;
    use crate::geometry::rect::Rect;

    fn assert_near(actual: Point, expected: Point) {
        assert!(
            (actual.x - expected.x).abs() < 1e-4 && (actual.y - expected.y).abs() < 1e-4,
            "{actual:?} is not {expected:?}"
        );
    }

    #[test]
    fn a_translation_moves_a_point() {
        assert_near(
            Transform::translation(10.0, -5.0).apply(Point { x: 1.0, y: 2.0 }),
            Point { x: 11.0, y: -3.0 },
        );
    }

    #[test]
    fn a_quarter_turn_clockwise_sends_right_to_down() {
        assert_near(
            Transform::rotation(90.0).apply(Point { x: 1.0, y: 0.0 }),
            Point { x: 0.0, y: 1.0 },
        );
    }

    #[test]
    fn then_applies_the_first_transform_before_the_second() {
        let scale_then_move = Transform::scaling(2.0, 2.0).then(&Transform::translation(10.0, 0.0));

        assert_near(
            scale_then_move.apply(Point { x: 1.0, y: 1.0 }),
            Point { x: 12.0, y: 2.0 },
        );
    }

    #[test]
    fn the_inverse_undoes_the_transform() {
        let transform = Transform::rotation(30.0)
            .then(&Transform::scaling(2.0, 0.5))
            .then(&Transform::translation(7.0, -3.0));
        let point = Point { x: 4.0, y: 9.0 };

        let back = transform
            .inverse()
            .expect("the transform is invertible")
            .apply(transform.apply(point));

        assert_near(back, point);
    }

    #[test]
    fn translations_far_from_the_origin_cancel_out_to_the_pixel() {
        let far_down = 16_777_217.0;
        let back_up = Transform::translation(0.0, far_down).then(&Transform::translation(0.0, 9.0 - far_down));

        assert_eq!(back_up.apply(Point { x: 0.0, y: 1.0 }), Point { x: 0.0, y: 10.0 });
    }

    #[test]
    fn a_collapsing_transform_has_no_inverse() {
        assert_eq!(Transform::scaling(0.0, 1.0).inverse(), None);
    }

    #[test]
    fn the_bounds_of_a_rotated_square_hold_its_corners() {
        let rotated = Transform::rotation(45.0);
        let square = Rect::new(-1.0, -1.0, 2.0, 2.0);

        let bounds = rotated.bounds_of(&square);

        let half_diagonal = 2.0_f32.sqrt();
        assert!((bounds.x + half_diagonal).abs() < 1e-4 && (bounds.width - 2.0 * half_diagonal).abs() < 1e-4);
    }
}
