use morok_style::{Theme, TransformFunction};

use crate::geometry::rect::Rect;
use crate::geometry::transform::Transform;
use crate::motion::decomposed_transform::DecomposedTransform;
use crate::style::resolved_length::ResolvedLength;

/// One step of a resolved transform list: a function with its tokens resolved and its
/// percentages kept for the box, or the blend of two lists whose functions do not pair up.
///
/// A blend is worked out once the box is known: the `from` steps that follow it are one list and
/// the `to` steps after them the other, each of which may hold blends of its own.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum TransformStep {
    /// Percentages of the border box.
    Translate(ResolvedLength, ResolvedLength),
    Scale(f32, f32),
    /// Degrees clockwise.
    Rotate(f32),
    Blend {
        progress: f32,
        from: u16,
        to: u16,
    },
}

impl TransformStep {
    /// A function of a `transform` declaration as the step of the primitive it is a form of.
    pub(crate) fn of(function: &TransformFunction, theme: &Theme) -> Self {
        match function {
            TransformFunction::Translate(x, y) => {
                Self::Translate(ResolvedLength::of(x, theme), ResolvedLength::of(y, theme))
            }
            TransformFunction::TranslateX(x) => Self::Translate(ResolvedLength::of(x, theme), ResolvedLength::ZERO),
            TransformFunction::TranslateY(y) => Self::Translate(ResolvedLength::ZERO, ResolvedLength::of(y, theme)),
            TransformFunction::Scale(x, y) => Self::Scale(*x, *y),
            TransformFunction::ScaleX(x) => Self::Scale(*x, 1.0),
            TransformFunction::ScaleY(y) => Self::Scale(1.0, *y),
            TransformFunction::Rotate(degrees) => Self::Rotate(*degrees),
        }
    }

    /// The step that leaves everything in place, of the same primitive; none for a blend.
    fn identity_like(self) -> Option<Self> {
        match self {
            Self::Translate(..) => Some(Self::Translate(ResolvedLength::ZERO, ResolvedLength::ZERO)),
            Self::Scale(..) => Some(Self::Scale(1.0, 1.0)),
            Self::Rotate(_) => Some(Self::Rotate(0.0)),
            Self::Blend { .. } => None,
        }
    }

    /// The step `progress` of the way to `other` when both are of the same primitive.
    fn paired_with(self, other: Self, progress: f32) -> Option<Self> {
        let between = |from: f32, to: f32| from + (to - from) * progress;
        let length = |from: ResolvedLength, to: ResolvedLength| ResolvedLength {
            pixels: between(from.pixels, to.pixels),
            fraction: match (from.fraction, to.fraction) {
                (None, None) => None,
                (from, to) => Some(between(from.unwrap_or(0.0), to.unwrap_or(0.0))),
            },
        };
        match (self, other) {
            (Self::Translate(from_x, from_y), Self::Translate(to_x, to_y)) => {
                Some(Self::Translate(length(from_x, to_x), length(from_y, to_y)))
            }
            (Self::Scale(from_x, from_y), Self::Scale(to_x, to_y)) => {
                Some(Self::Scale(between(from_x, to_x), between(from_y, to_y)))
            }
            (Self::Rotate(from), Self::Rotate(to)) => Some(Self::Rotate(between(from, to))),
            _ => None,
        }
    }
}

/// The resolved steps of the functions of a `transform` declaration.
pub(crate) fn resolve_into(functions: &[TransformFunction], theme: &Theme, steps: &mut Vec<TransformStep>) {
    steps.clear();
    steps.extend(functions.iter().map(|function| TransformStep::of(function, theme)));
}

/// The transform the steps make on a box laid out at `border_box`, about the origin of the
/// coordinates the box is laid out in.
pub(crate) fn transform_of(steps: &[TransformStep], border_box: &Rect) -> Transform {
    let mut combined = Transform::IDENTITY;
    let mut position = 0;
    while position < steps.len() {
        let (matrix, length) = item_transform(&steps[position..], border_box);
        combined = matrix.then(&combined);
        position += length;
    }
    combined
}

/// The transform of the first item of `steps` and the number of steps it takes.
fn item_transform(steps: &[TransformStep], border_box: &Rect) -> (Transform, usize) {
    let matrix = match steps[0] {
        TransformStep::Translate(x, y) => {
            Transform::translation(x.against(border_box.width), y.against(border_box.height))
        }
        TransformStep::Scale(x, y) => Transform::scaling(x, y),
        TransformStep::Rotate(degrees) => Transform::rotation(degrees),
        TransformStep::Blend { progress, from, to } => {
            let (from, to) = (usize::from(from), usize::from(to));
            let from_transform = transform_of(&steps[1..=from], border_box);
            let to_transform = transform_of(&steps[1 + from..=from + to], border_box);
            let blended = blended_matrices(&from_transform, &to_transform, f64::from(progress));
            return (blended, 1 + from + to);
        }
    };
    (matrix, 1)
}

/// Two matrices interpolated through their parts; one that cannot be taken apart flips to the
/// other half way.
fn blended_matrices(from: &Transform, to: &Transform, progress: f64) -> Transform {
    match (DecomposedTransform::of(from), DecomposedTransform::of(to)) {
        (Some(from), Some(to)) => from.interpolated(&to, progress).recomposed(),
        _ if progress < 0.5 => *from,
        _ => *to,
    }
}

/// Writes into `blended` the transform `progress` of the way from `from` to `to`. Functions of
/// the same primitive at the same position are interpolated one by one, a missing function
/// standing in as the identity; from the first pair that differs on, the rest of both lists is
/// blended as matrices once the box is known.
pub(crate) fn blend_into(
    from: &[TransformStep],
    to: &[TransformStep],
    progress: f32,
    blended: &mut Vec<TransformStep>,
) {
    blended.clear();
    let (mut from_position, mut to_position) = (0, 0);
    loop {
        let from_item = from.get(from_position).copied();
        let to_item = to.get(to_position).copied();
        let paired = match (from_item, to_item) {
            (None, None) => return,
            (Some(from_step), Some(to_step)) => from_step.paired_with(to_step, progress),
            (Some(from_step), None) => from_step
                .identity_like()
                .and_then(|identity| from_step.paired_with(identity, progress)),
            (None, Some(to_step)) => to_step
                .identity_like()
                .and_then(|identity| identity.paired_with(to_step, progress)),
        };
        let Some(step) = paired else {
            break;
        };
        blended.push(step);
        from_position += from_item.map_or(0, |_| 1);
        to_position += to_item.map_or(0, |_| 1);
    }
    let from_rest = &from[from_position..];
    let to_rest = &to[to_position..];
    blended.push(TransformStep::Blend {
        progress,
        from: u16::try_from(from_rest.len()).expect("a transform list holds fewer than 2^16 steps"),
        to: u16::try_from(to_rest.len()).expect("a transform list holds fewer than 2^16 steps"),
    });
    blended.extend_from_slice(from_rest);
    blended.extend_from_slice(to_rest);
}

#[cfg(test)]
mod tests {
    use super::{TransformStep, blend_into, transform_of};
    use crate::geometry::point::Point;
    use crate::geometry::rect::Rect;
    use crate::geometry::transform::Transform;
    use crate::style::resolved_length::ResolvedLength;

    const BOX: Rect = Rect {
        x: 0.0,
        y: 0.0,
        width: 100.0,
        height: 40.0,
    };

    fn moved(x: f32) -> TransformStep {
        TransformStep::Translate(ResolvedLength::pixels(x), ResolvedLength::ZERO)
    }

    #[test]
    fn functions_of_one_primitive_are_interpolated_one_by_one() {
        let mut blended = Vec::new();

        blend_into(
            &[moved(0.0), TransformStep::Rotate(0.0)],
            &[moved(40.0), TransformStep::Rotate(90.0)],
            0.25,
            &mut blended,
        );

        assert_eq!(blended, [moved(10.0), TransformStep::Rotate(22.5)]);
    }

    #[test]
    fn a_missing_function_stands_in_as_the_identity() {
        let mut blended = Vec::new();

        blend_into(&[], &[TransformStep::Scale(2.0, 3.0)], 0.5, &mut blended);

        assert_eq!(blended, [TransformStep::Scale(1.5, 2.0)]);
    }

    #[test]
    fn lists_that_differ_blend_their_rest_as_matrices_on_the_box() {
        let mut blended = Vec::new();
        blend_into(
            &[moved(0.0), TransformStep::Rotate(90.0)],
            &[moved(20.0), TransformStep::Scale(2.0, 2.0)],
            0.5,
            &mut blended,
        );

        assert_eq!(blended[0], moved(10.0));
        let transform = transform_of(&blended, &BOX);
        let corner = transform.apply(Point { x: 1.0, y: 0.0 });
        let expected_scale = 1.5_f32;
        let expected_angle = 45.0_f32.to_radians();
        assert!(
            (corner.x - (10.0 + expected_scale * expected_angle.cos())).abs() < 1e-4
                && (corner.y - expected_scale * expected_angle.sin()).abs() < 1e-4,
            "{corner:?}"
        );
    }

    #[test]
    fn a_percentage_is_of_the_box_the_steps_are_applied_to() {
        let half_width = TransformStep::Translate(
            ResolvedLength {
                pixels: 0.0,
                fraction: Some(0.5),
            },
            ResolvedLength::ZERO,
        );

        assert_eq!(transform_of(&[half_width], &BOX), Transform::translation(50.0, 0.0));
    }
}
