/// How strongly the edge resists a gesture that pulls the content past its limit: the smaller,
/// the stiffer.
const RESISTANCE: f32 = 0.55;

/// The offset an axis shows when a gesture would put it at `unresisted` without resistance:
/// within `0..=limit` the same, past an end only as far as the rubber band lets it go, which
/// never reaches `dimension`, the length of the viewport.
pub(crate) fn shown_offset(unresisted: f32, limit: f32, dimension: f32) -> f32 {
    if unresisted < 0.0 {
        -stretch(-unresisted, dimension)
    } else if unresisted > limit {
        limit + stretch(unresisted - limit, dimension)
    } else {
        unresisted
    }
}

/// The unresisted offset that shows `shown`: the inverse of [`shown_offset`].
pub(crate) fn unresisted_offset(shown: f32, limit: f32, dimension: f32) -> f32 {
    if shown < 0.0 {
        -pull(-shown, dimension)
    } else if shown > limit {
        limit + pull(shown - limit, dimension)
    } else {
        shown
    }
}

/// How far past the end the content shows for a pull of `distance`.
fn stretch(distance: f32, dimension: f32) -> f32 {
    if dimension <= 0.0 {
        return 0.0;
    }
    (1.0 - 1.0 / (distance * RESISTANCE / dimension + 1.0)) * dimension
}

/// The pull that shows the content `stretched` past the end.
fn pull(stretched: f32, dimension: f32) -> f32 {
    if dimension <= 0.0 {
        return 0.0;
    }
    let share = (stretched / dimension).min(MOST_OF_THE_VIEWPORT);
    (1.0 / (1.0 - share) - 1.0) * dimension / RESISTANCE
}

/// The farthest a stretch is taken back from: the rubber band only nears the whole viewport.
const MOST_OF_THE_VIEWPORT: f32 = 0.99;

#[cfg(test)]
mod tests {
    use super::{shown_offset, unresisted_offset};

    #[test]
    fn within_the_limits_the_offset_is_unresisted_and_past_them_it_stretches_less_and_less() {
        assert!((shown_offset(120.0, 400.0, 200.0) - 120.0).abs() < 1e-4);
        let near = shown_offset(450.0, 400.0, 200.0) - 400.0;
        let far = shown_offset(800.0, 400.0, 200.0) - 400.0;
        assert!(near > 0.0 && near < 50.0, "the edge resists");
        assert!(
            far > near && far < 200.0,
            "the stretch grows and never reaches the viewport"
        );
        assert!(shown_offset(-50.0, 400.0, 200.0) < 0.0);
    }

    #[test]
    fn the_unresisted_offset_takes_a_stretch_back_to_its_pull() {
        for unresisted in [-300.0, -20.0, 0.0, 250.0, 430.0, 900.0] {
            let shown = shown_offset(unresisted, 400.0, 200.0);
            assert!((unresisted_offset(shown, 400.0, 200.0) - unresisted).abs() < 0.01);
        }
    }
}
