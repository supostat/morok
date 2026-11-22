use crate::geometry::point::Point;
use crate::geometry::rect::Rect;
use crate::geometry::size::Size;
use crate::overlay::overlay_alignment::OverlayAlignment;
use crate::overlay::overlay_side::OverlaySide;
use crate::overlay::placed_overlay::PlacedOverlay;

/// How an overlay is placed against its anchor.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct OverlayPlacement {
    /// The side of the anchor the overlay lies on while there is room for it there.
    pub side: OverlaySide,
    pub alignment: OverlayAlignment,
    /// The distance between the anchor and the overlay.
    pub gap: f32,
    /// How far the overlay is moved along the edge of the anchor from where the alignment
    /// puts it: to the right or downwards when positive.
    pub shift: f32,
}

impl OverlayPlacement {
    /// Places an overlay of `size` against `anchor` inside `bounds`, all three in the same
    /// coordinates with `y` growing downwards.
    ///
    /// The overlay lies on the side asked for when it fits between the anchor and the edge of
    /// the bounds there, flips to the opposite side when it fits only there, and takes the
    /// roomier of the two when it fits on neither. It is then moved the least that keeps it
    /// inside the bounds; an overlay larger than the bounds starts at their start. How far
    /// that moved it along the edge of the anchor is reported with it.
    pub fn place(&self, anchor: Rect, size: Size, bounds: Rect) -> PlacedOverlay {
        let lies_on = self.side_with_room(anchor, size, bounds);
        let origin = self.origin_on(lies_on, anchor, size);
        let kept = Point {
            x: kept_inside(origin.x, size.width, bounds.x, bounds.right()),
            y: kept_inside(origin.y, size.height, bounds.y, bounds.bottom()),
        };
        let edge_shift = if lies_on.is_vertical() {
            kept.x - origin.x
        } else {
            kept.y - origin.y
        };
        PlacedOverlay {
            frame: Rect {
                x: kept.x,
                y: kept.y,
                width: size.width,
                height: size.height,
            },
            side: lies_on,
            edge_shift,
        }
    }

    fn side_with_room(&self, anchor: Rect, size: Size, bounds: Rect) -> OverlaySide {
        let needed = if self.side.is_vertical() {
            size.height
        } else {
            size.width
        };
        let asked = self.room_on(self.side, anchor, bounds);
        let opposite = self.room_on(self.side.opposite(), anchor, bounds);
        if asked >= needed || asked >= opposite {
            self.side
        } else {
            self.side.opposite()
        }
    }

    /// How much of the bounds lies past the anchor and the gap on the side.
    fn room_on(&self, side: OverlaySide, anchor: Rect, bounds: Rect) -> f32 {
        match side {
            OverlaySide::Below => bounds.bottom() - anchor.bottom() - self.gap,
            OverlaySide::Above => anchor.y - bounds.y - self.gap,
            OverlaySide::Right => bounds.right() - anchor.right() - self.gap,
            OverlaySide::Left => anchor.x - bounds.x - self.gap,
        }
    }

    fn origin_on(&self, lies_on: OverlaySide, anchor: Rect, size: Size) -> Point {
        let along_width = self.aligned(anchor.x, anchor.width, size.width);
        let along_height = self.aligned(anchor.y, anchor.height, size.height);
        match lies_on {
            OverlaySide::Below => Point {
                x: along_width,
                y: anchor.bottom() + self.gap,
            },
            OverlaySide::Above => Point {
                x: along_width,
                y: anchor.y - self.gap - size.height,
            },
            OverlaySide::Right => Point {
                x: anchor.right() + self.gap,
                y: along_height,
            },
            OverlaySide::Left => Point {
                x: anchor.x - self.gap - size.width,
                y: along_height,
            },
        }
    }

    /// Where an overlay of `length` starts along an edge of the anchor that starts at
    /// `anchor_start` and is `anchor_length` long.
    fn aligned(&self, anchor_start: f32, anchor_length: f32, length: f32) -> f32 {
        let aligned = match self.alignment {
            OverlayAlignment::Start => anchor_start,
            OverlayAlignment::Center => anchor_start + (anchor_length - length) / 2.0,
            OverlayAlignment::End => anchor_start + anchor_length - length,
        };
        aligned + self.shift
    }
}

/// The start nearest to `start` that keeps `length` between `bounds_start` and `bounds_end`.
fn kept_inside(start: f32, length: f32, bounds_start: f32, bounds_end: f32) -> f32 {
    start.min(bounds_end - length).max(bounds_start)
}

#[cfg(test)]
mod tests {
    use super::OverlayPlacement;
    use crate::geometry::rect::Rect;
    use crate::geometry::size::Size;
    use crate::overlay::overlay_alignment::OverlayAlignment;
    use crate::overlay::overlay_side::OverlaySide;
    use crate::overlay::placed_overlay::PlacedOverlay;

    const SCREEN: Rect = Rect::new(0.0, 0.0, 1000.0, 800.0);
    const PANEL: Size = Size {
        width: 200.0,
        height: 120.0,
    };

    fn placement(side: OverlaySide, alignment: OverlayAlignment) -> OverlayPlacement {
        OverlayPlacement {
            side,
            alignment,
            gap: 4.0,
            shift: 0.0,
        }
    }

    #[test]
    fn an_overlay_lies_on_the_side_asked_for_with_the_gap_between() {
        let anchor = Rect::new(300.0, 200.0, 80.0, 28.0);

        let below = placement(OverlaySide::Below, OverlayAlignment::Start).place(anchor, PANEL, SCREEN);
        let above = placement(OverlaySide::Above, OverlayAlignment::End).place(anchor, PANEL, SCREEN);
        let right = placement(OverlaySide::Right, OverlayAlignment::Center).place(anchor, PANEL, SCREEN);
        let left = placement(OverlaySide::Left, OverlayAlignment::Start).place(anchor, PANEL, SCREEN);

        assert_eq!(
            (below.frame, below.side),
            (Rect::new(300.0, 232.0, 200.0, 120.0), OverlaySide::Below)
        );
        assert_eq!(
            (above.frame, above.side),
            (Rect::new(180.0, 76.0, 200.0, 120.0), OverlaySide::Above)
        );
        assert_eq!(
            (right.frame, right.side),
            (Rect::new(384.0, 154.0, 200.0, 120.0), OverlaySide::Right)
        );
        assert_eq!(
            (left.frame, left.side),
            (Rect::new(96.0, 200.0, 200.0, 120.0), OverlaySide::Left)
        );
    }

    #[test]
    fn an_overlay_with_no_room_on_its_side_flips_to_the_opposite_one() {
        let near_the_bottom = Rect::new(300.0, 740.0, 80.0, 28.0);
        let near_the_right = Rect::new(900.0, 300.0, 80.0, 28.0);

        let flipped_up = placement(OverlaySide::Below, OverlayAlignment::Start).place(near_the_bottom, PANEL, SCREEN);
        let flipped_left = placement(OverlaySide::Right, OverlayAlignment::Start).place(near_the_right, PANEL, SCREEN);

        assert_eq!(
            (flipped_up.frame, flipped_up.side),
            (Rect::new(300.0, 616.0, 200.0, 120.0), OverlaySide::Above)
        );
        assert_eq!(
            (flipped_left.frame, flipped_left.side),
            (Rect::new(696.0, 300.0, 200.0, 120.0), OverlaySide::Left)
        );
    }

    #[test]
    fn an_overlay_that_fits_on_neither_side_takes_the_roomier_one_and_stays_inside() {
        let tall = Size {
            width: 200.0,
            height: 500.0,
        };
        let below_the_middle = Rect::new(300.0, 450.0, 80.0, 28.0);

        let placed = placement(OverlaySide::Below, OverlayAlignment::Start).place(below_the_middle, tall, SCREEN);

        assert_eq!(placed.side, OverlaySide::Above, "there is more room above the anchor");
        assert_eq!(
            placed.frame,
            Rect::new(300.0, 0.0, 200.0, 500.0),
            "and it starts at the top of the bounds"
        );
        assert_eq!(
            placed.edge_shift.to_bits(),
            0.0_f32.to_bits(),
            "a move across its side is no move along it"
        );
    }

    #[test]
    fn an_overlay_is_moved_along_the_edge_to_stay_inside_the_bounds_and_reports_how_far() {
        let below = placement(OverlaySide::Below, OverlayAlignment::Start);
        let right = placement(OverlaySide::Right, OverlayAlignment::Start);

        let at_the_right_edge = below.place(Rect::new(950.0, 200.0, 40.0, 28.0), PANEL, SCREEN);
        let at_the_left_edge = placement(OverlaySide::Below, OverlayAlignment::End).place(
            Rect::new(10.0, 200.0, 40.0, 28.0),
            PANEL,
            SCREEN,
        );
        let at_the_bottom_edge = right.place(Rect::new(300.0, 760.0, 40.0, 28.0), PANEL, SCREEN);
        let at_the_top_edge = placement(OverlaySide::Right, OverlayAlignment::End).place(
            Rect::new(300.0, 10.0, 40.0, 28.0),
            PANEL,
            SCREEN,
        );

        assert_eq!(
            at_the_right_edge,
            PlacedOverlay {
                frame: Rect::new(800.0, 232.0, 200.0, 120.0),
                side: OverlaySide::Below,
                edge_shift: -150.0,
            }
        );
        assert_eq!(
            at_the_left_edge,
            PlacedOverlay {
                frame: Rect::new(0.0, 232.0, 200.0, 120.0),
                side: OverlaySide::Below,
                edge_shift: 150.0,
            }
        );
        assert_eq!(
            at_the_bottom_edge,
            PlacedOverlay {
                frame: Rect::new(344.0, 680.0, 200.0, 120.0),
                side: OverlaySide::Right,
                edge_shift: -80.0,
            }
        );
        assert_eq!(
            at_the_top_edge,
            PlacedOverlay {
                frame: Rect::new(344.0, 0.0, 200.0, 120.0),
                side: OverlaySide::Right,
                edge_shift: 82.0,
            }
        );
    }

    #[test]
    fn an_overlay_that_fits_where_its_placement_puts_it_is_not_moved_along_its_side() {
        let anchor = Rect::new(300.0, 200.0, 80.0, 28.0);

        let placed = placement(OverlaySide::Below, OverlayAlignment::Center).place(anchor, PANEL, SCREEN);

        assert_eq!(
            placed,
            PlacedOverlay {
                frame: Rect::new(240.0, 232.0, 200.0, 120.0),
                side: OverlaySide::Below,
                edge_shift: 0.0,
            }
        );
    }

    #[test]
    fn a_shift_moves_the_overlay_along_the_edge_of_the_anchor() {
        let anchor = Rect::new(300.0, 200.0, 80.0, 28.0);
        let shifted = OverlayPlacement {
            side: OverlaySide::Right,
            alignment: OverlayAlignment::Start,
            gap: 0.0,
            shift: -5.0,
        };

        assert_eq!(
            shifted.place(anchor, PANEL, SCREEN).frame,
            Rect::new(380.0, 195.0, 200.0, 120.0)
        );
    }
}
