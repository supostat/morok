use crate::geometry::point::Point;
use crate::geometry::size::Size;
use crate::motion::spring_id::SpringId;
use crate::scroll::rubber_band::{shown_offset, unresisted_offset};
use crate::scroll::scroll_bars::ScrollBars;
use crate::scroll::scroll_phase::ScrollPhase;

/// What a tree keeps about an element of the `Scroll` role: how far its content is scrolled,
/// how far it can be, and the gesture and springs that move it.
#[derive(Debug, Default)]
pub(crate) struct ScrollState {
    /// Where the content is now: within the limits, or past them while a gesture pulls it there
    /// or a spring takes it back.
    pub(crate) offset: Point,
    /// Where the gesture under way would put the content without the resistance of the edges.
    unresisted: Point,
    /// The offset the last frame showed: what is painted and hit.
    pub(crate) shown: Point,
    /// An offset asked for before the limits it is kept within were known; the next frame
    /// takes it.
    pub(crate) requested: Option<Point>,
    /// The largest offset of each axis as the last layout found it: zero on an axis that does
    /// not scroll or whose content fits.
    pub(crate) limit: Point,
    /// The size of the padding box, which the content scrolls in.
    pub(crate) viewport: Size,
    pub(crate) bars: Option<ScrollBars>,
    /// The springs taking an offset past a limit back to it, by axis.
    pub(crate) return_x: Option<SpringId>,
    pub(crate) return_y: Option<SpringId>,
    /// Whether a gesture or its momentum is under way.
    pub(crate) is_in_gesture: bool,
    /// Whether the offset moved in the last frame, so that its scroll bars show.
    pub(crate) moved_in_last_frame: bool,
    /// What the scrolls of each axis moved the content by that its offset cannot tell: far
    /// down a long content an offset has steps of more than a pixel. It joins the next scroll,
    /// so that small scrolls add up.
    carried: [f64; 2],
}

impl ScrollState {
    /// The offset nearest to `offset` within the limits.
    pub(crate) fn clamped(&self, offset: Point) -> Point {
        Point {
            x: offset.x.clamp(0.0, self.limit.x),
            y: offset.y.clamp(0.0, self.limit.y),
        }
    }

    pub(crate) fn has_return(&self) -> bool {
        self.return_x.is_some() || self.return_y.is_some()
    }

    /// Moves the content to `offset` within the limits, ending what moved it before.
    pub(crate) fn move_to(&mut self, offset: Point) {
        self.offset = self.clamped(offset);
        self.unresisted = self.offset;
        self.carried = [0.0; 2];
    }

    /// Moves the content by a scroll of the phase: a scroll of no gesture stops at the limits,
    /// a gesture pulls the content past them against the resistance of the edge. The axes that
    /// do not scroll keep their offset.
    pub(crate) fn scroll(&mut self, delta: Point, phase: ScrollPhase, scrolls_x: bool, scrolls_y: bool) {
        let delta = Point {
            x: if scrolls_x { delta.x } else { 0.0 },
            y: if scrolls_y { delta.y } else { 0.0 },
        };
        let [carried_x, carried_y] = &mut self.carried;
        if !phase.is_of_gesture() {
            let goal = Point {
                x: advanced(self.offset.x, delta.x, carried_x),
                y: advanced(self.offset.y, delta.y, carried_y),
            };
            let [left_over_x, left_over_y] = self.carried;
            self.move_to(goal);
            // An axis stopped by its limit has nothing left to carry.
            self.carried = [
                if self.offset.x.to_bits() == goal.x.to_bits() {
                    left_over_x
                } else {
                    0.0
                },
                if self.offset.y.to_bits() == goal.y.to_bits() {
                    left_over_y
                } else {
                    0.0
                },
            ];
            return;
        }
        self.unresisted.x = advanced(self.unresisted.x, delta.x, carried_x);
        self.unresisted.y = advanced(self.unresisted.y, delta.y, carried_y);
        self.offset = Point {
            x: shown_offset(self.unresisted.x, self.limit.x, self.viewport.width),
            y: shown_offset(self.unresisted.y, self.limit.y, self.viewport.height),
        };
    }

    /// Takes the gesture back from where the content is now, past a limit or not: a spring
    /// that was taking it back has been stopped.
    pub(crate) fn resume_gesture(&mut self) {
        self.carried = [0.0; 2];
        self.unresisted = Point {
            x: unresisted_offset(self.offset.x, self.limit.x, self.viewport.width),
            y: unresisted_offset(self.offset.y, self.limit.y, self.viewport.height),
        };
    }
}

/// The offset `delta` further than `offset`, as near as an offset of its magnitude tells it;
/// what is left over is carried to the next scroll of the axis.
fn advanced(offset: f32, delta: f32, carried: &mut f64) -> f32 {
    let precise = f64::from(offset) + f64::from(delta) + *carried;
    #[expect(
        clippy::cast_possible_truncation,
        reason = "offsets are single precision; the rest is carried"
    )]
    let advanced = precise as f32;
    *carried = precise - f64::from(advanced);
    advanced
}
