use crate::check::Check;

const LAST_ROW: &str = "last row";
/// How far from the bottom of the viewport the last row may end: boxes are placed on device
/// pixels, so a row that ends where it should ends there exactly.
const PLACEMENT_TOLERANCE: f32 = 0.01;

/// Holds where a scroll through every row of a table ended against where it must end: with the
/// last row in a slot whose bottom lies at the bottom of the viewport. `below_viewport` is how
/// far below that the last row ends, or none when no slot shows the last row. A scroll that did
/// not come this far measured frames that bound no rows.
pub(crate) fn judge_end(below_viewport: Option<f32>) -> Check {
    Check {
        subject: LAST_ROW,
        measured: below_viewport.map_or_else(
            || String::from("not shown"),
            |distance| format!("ends {distance:.2} px below"),
        ),
        budget: String::from("ends at the bottom"),
        holds: below_viewport.is_some_and(|distance| distance.abs() < PLACEMENT_TOLERANCE),
    }
}

#[cfg(test)]
mod tests {
    use super::judge_end;

    #[test]
    fn a_scroll_that_ends_with_the_last_row_at_the_bottom_of_the_viewport_holds() {
        assert!(judge_end(Some(0.0)).holds);
    }

    #[test]
    fn a_scroll_that_never_showed_the_last_row_fails() {
        let check = judge_end(None);

        assert!(!check.holds);
        assert_eq!((check.subject, check.measured.as_str()), ("last row", "not shown"));
    }

    #[test]
    fn a_last_row_placed_off_its_place_fails() {
        assert!(
            !judge_end(Some(2.0)).holds,
            "a row placed with the precision of a far offset"
        );
        assert!(
            !judge_end(Some(-26.0)).holds,
            "a scroll that stopped a row short of the end"
        );
    }
}
