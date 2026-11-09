use crate::geometry::point::Point;
use crate::geometry::sides::Sides;
use crate::geometry::size::Size;

/// An axis-aligned rectangle in CSS pixels, in the coordinates of its surface.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl Rect {
    pub const ZERO: Self = Self::new(0.0, 0.0, 0.0, 0.0);

    pub const fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self { x, y, width, height }
    }

    /// The top left corner.
    pub fn origin(&self) -> Point {
        Point { x: self.x, y: self.y }
    }

    pub fn size(&self) -> Size {
        Size {
            width: self.width,
            height: self.height,
        }
    }

    pub fn right(&self) -> f32 {
        self.x + self.width
    }

    pub fn bottom(&self) -> f32 {
        self.y + self.height
    }

    /// Whether the rectangle covers no area.
    pub fn is_empty(&self) -> bool {
        self.width <= 0.0 || self.height <= 0.0
    }

    pub fn contains(&self, other: &Self) -> bool {
        self.x <= other.x && self.y <= other.y && self.right() >= other.right() && self.bottom() >= other.bottom()
    }

    /// Whether the two rectangles share any area; touching edges do not count.
    pub fn intersects(&self, other: &Self) -> bool {
        self.x < other.right() && other.x < self.right() && self.y < other.bottom() && other.y < self.bottom()
    }

    /// The smallest rectangle that contains both.
    #[must_use]
    pub fn union(&self, other: &Self) -> Self {
        let x = self.x.min(other.x);
        let y = self.y.min(other.y);
        Self {
            x,
            y,
            width: self.right().max(other.right()) - x,
            height: self.bottom().max(other.bottom()) - y,
        }
    }

    /// The rectangle with every side moved outwards by its distance; a negative distance moves the side inwards.
    #[must_use]
    pub fn expanded(&self, distances: Sides<f32>) -> Self {
        Self {
            x: self.x - distances.left,
            y: self.y - distances.top,
            width: self.width + distances.left + distances.right,
            height: self.height + distances.top + distances.bottom,
        }
    }
}
