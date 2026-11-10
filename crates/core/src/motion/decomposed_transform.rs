use crate::geometry::transform::Transform;

/// A 2D affine transform taken apart into a translation, a rotation, a skew and a scale, in the
/// order they apply last to first: the form two matrices are interpolated in.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct DecomposedTransform {
    translate_x: f64,
    translate_y: f64,
    /// Degrees clockwise.
    angle: f64,
    /// The shear of the vertical axis towards the horizontal one.
    skew: f64,
    scale_x: f64,
    scale_y: f64,
}

impl DecomposedTransform {
    /// None when the transform collapses the plane onto a line or a point.
    pub(crate) fn of(transform: &Transform) -> Option<Self> {
        let [a, b, c, d] = [transform.a, transform.b, transform.c, transform.d].map(f64::from);
        let scale_x = a.hypot(b);
        if scale_x == 0.0 || a * d - b * c == 0.0 {
            return None;
        }
        let (cosine, sine) = (a / scale_x, b / scale_x);
        let scale_y = -sine * c + cosine * d;
        Some(Self {
            translate_x: f64::from(transform.tx),
            translate_y: f64::from(transform.ty),
            angle: sine.atan2(cosine).to_degrees(),
            skew: (cosine * c + sine * d) / scale_y,
            scale_x,
            scale_y,
        })
    }

    /// The transform `progress` of the way from `self` to `other`, turning the shorter way round.
    pub(crate) fn interpolated(&self, other: &Self, progress: f64) -> Self {
        let mut from_angle = self.angle;
        let difference = other.angle - from_angle;
        if difference > 180.0 {
            from_angle += 360.0;
        } else if difference < -180.0 {
            from_angle -= 360.0;
        }
        let between = |from: f64, to: f64| from + (to - from) * progress;
        Self {
            translate_x: between(self.translate_x, other.translate_x),
            translate_y: between(self.translate_y, other.translate_y),
            angle: between(from_angle, other.angle),
            skew: between(self.skew, other.skew),
            scale_x: between(self.scale_x, other.scale_x),
            scale_y: between(self.scale_y, other.scale_y),
        }
    }

    pub(crate) fn recomposed(&self) -> Transform {
        let (sine, cosine) = self.angle.to_radians().sin_cos();
        let skewed_x = self.skew * self.scale_y;
        Transform {
            a: (cosine * self.scale_x) as f32,
            b: (sine * self.scale_x) as f32,
            c: (cosine * skewed_x - sine * self.scale_y) as f32,
            d: (sine * skewed_x + cosine * self.scale_y) as f32,
            tx: self.translate_x as f32,
            ty: self.translate_y as f32,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::DecomposedTransform;
    use crate::geometry::transform::Transform;

    fn assert_near(actual: Transform, expected: Transform) {
        let pairs = [
            (actual.a, expected.a),
            (actual.b, expected.b),
            (actual.c, expected.c),
            (actual.d, expected.d),
            (actual.tx, expected.tx),
            (actual.ty, expected.ty),
        ];
        assert!(
            pairs.iter().all(|(left, right)| (left - right).abs() < 1e-4),
            "{actual:?} is not {expected:?}"
        );
    }

    #[test]
    fn a_transform_is_recomposed_from_its_parts() {
        let transform = Transform::scaling(2.0, 0.5)
            .then(&Transform::rotation(30.0))
            .then(&Transform::translation(5.0, -7.0));

        let parts = DecomposedTransform::of(&transform).expect("the transform is invertible");

        assert_near(parts.recomposed(), transform);
    }

    #[test]
    fn a_skew_survives_the_round_trip() {
        let skewed = Transform::rotation(20.0).then(&Transform::scaling(3.0, 1.0));

        let parts = DecomposedTransform::of(&skewed).expect("the transform is invertible");

        assert_near(parts.recomposed(), skewed);
    }

    #[test]
    fn rotations_meet_the_shorter_way_round() {
        let from = DecomposedTransform::of(&Transform::rotation(170.0)).expect("a rotation is invertible");
        let to = DecomposedTransform::of(&Transform::rotation(-170.0)).expect("a rotation is invertible");

        assert_near(from.interpolated(&to, 0.5).recomposed(), Transform::rotation(180.0));
    }

    #[test]
    fn a_collapsed_transform_cannot_be_taken_apart() {
        assert_eq!(DecomposedTransform::of(&Transform::scaling(0.0, 1.0)), None);
    }
}
