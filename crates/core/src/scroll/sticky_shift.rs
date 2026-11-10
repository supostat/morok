use crate::geometry::point::Point;
use crate::geometry::rect::Rect;
use crate::geometry::sides::Sides;
use crate::style::resolved_length_or_auto::ResolvedLengthOrAuto;

/// Where a sticky element stands against the scroll of its nearest `Scroll` ancestor: its box
/// as layout placed it, its margins, the content box of its parent, which it never leaves, and
/// the scrollport — the padding box of the `Scroll` element — in the coordinates of the content.
#[derive(Clone, Copy, Debug)]
pub(crate) struct StickyPlace {
    pub(crate) border_box: Rect,
    pub(crate) margin: Sides<f32>,
    pub(crate) containing_block: Rect,
    pub(crate) scrollport: Rect,
}

impl StickyPlace {
    /// How far the element moves so that each edge with an inset stays that far inside the
    /// scrollport, as far as its margin box stays inside its containing block.
    pub(crate) fn shift(&self, insets: &Sides<ResolvedLengthOrAuto>) -> Point {
        Point {
            x: axis_shift(
                AxisSpan {
                    start: self.border_box.x,
                    end: self.border_box.right(),
                    margin_start: self.margin.left,
                    margin_end: self.margin.right,
                },
                (self.containing_block.x, self.containing_block.right()),
                (self.scrollport.x, self.scrollport.right()),
                (insets.left, insets.right),
            ),
            y: axis_shift(
                AxisSpan {
                    start: self.border_box.y,
                    end: self.border_box.bottom(),
                    margin_start: self.margin.top,
                    margin_end: self.margin.bottom,
                },
                (self.containing_block.y, self.containing_block.bottom()),
                (self.scrollport.y, self.scrollport.bottom()),
                (insets.top, insets.bottom),
            ),
        }
    }
}

/// The extent of the element along one axis with its margins.
#[derive(Clone, Copy, Debug)]
struct AxisSpan {
    start: f32,
    end: f32,
    margin_start: f32,
    margin_end: f32,
}

fn axis_shift(
    span: AxisSpan,
    (block_start, block_end): (f32, f32),
    (port_start, port_end): (f32, f32),
    (start_inset, end_inset): (ResolvedLengthOrAuto, ResolvedLengthOrAuto),
) -> f32 {
    let port_length = port_end - port_start;
    let mut shift = 0.0;
    if let ResolvedLengthOrAuto::Length(inset) = start_inset {
        let behind = port_start + inset.against(port_length) - span.start;
        let room = block_end - (span.end + span.margin_end);
        if behind > 0.0 {
            shift += behind.min(room.max(0.0));
        }
    }
    if let ResolvedLengthOrAuto::Length(inset) = end_inset {
        let ahead = span.end + shift - (port_end - inset.against(port_length));
        let room = span.start - span.margin_start - block_start;
        if ahead > 0.0 {
            shift -= ahead.min(room.max(0.0));
        }
    }
    shift
}
