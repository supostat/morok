use morok_core::Rect;

/// A rectangle of device pixels of a render target.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct PixelRect {
    pub(crate) x: usize,
    pub(crate) y: usize,
    pub(crate) width: usize,
    pub(crate) height: usize,
}

impl PixelRect {
    /// The pixels a rectangle of CSS pixels touches.
    pub(crate) fn covering(rect: &Rect, scale: f32) -> Self {
        Self::between(
            (rect.x * scale).floor(),
            (rect.y * scale).floor(),
            (rect.right() * scale).ceil(),
            (rect.bottom() * scale).ceil(),
        )
    }

    /// The rectangle of CSS pixels with every edge moved to the nearest pixel edge.
    pub(crate) fn nearest(rect: &Rect, scale: f32) -> Self {
        Self::between(
            (rect.x * scale).round(),
            (rect.y * scale).round(),
            (rect.right() * scale).round(),
            (rect.bottom() * scale).round(),
        )
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.width == 0 || self.height == 0
    }

    #[must_use]
    pub(crate) fn intersection(&self, other: &Self) -> Self {
        let left = self.x.max(other.x);
        let top = self.y.max(other.y);
        let right = (self.x + self.width).min(other.x + other.width);
        let bottom = (self.y + self.height).min(other.y + other.height);
        Self {
            x: left,
            y: top,
            width: right.saturating_sub(left),
            height: bottom.saturating_sub(top),
        }
    }

    /// Edges left of or above the target are pulled onto it: a float below zero casts to zero.
    fn between(left: f32, top: f32, right: f32, bottom: f32) -> Self {
        let [left, top, right, bottom] = [left, top, right, bottom].map(|edge| edge as usize);
        Self {
            x: left,
            y: top,
            width: right.saturating_sub(left),
            height: bottom.saturating_sub(top),
        }
    }
}

#[cfg(test)]
mod tests {
    use morok_core::Rect;

    use super::PixelRect;

    fn pixels(x: usize, y: usize, width: usize, height: usize) -> PixelRect {
        PixelRect { x, y, width, height }
    }

    #[test]
    fn covering_takes_every_pixel_the_rectangle_touches() {
        let rect = Rect::new(10.25, 20.0, 5.5, 4.25);

        assert_eq!(PixelRect::covering(&rect, 2.0), pixels(20, 40, 12, 9));
    }

    #[test]
    fn nearest_moves_every_edge_to_the_closest_pixel_edge() {
        let rect = Rect::new(10.2, 20.0, 5.5, 4.3);

        assert_eq!(PixelRect::nearest(&rect, 2.0), pixels(20, 40, 11, 9));
    }

    #[test]
    fn a_rectangle_left_of_the_target_starts_at_its_edge() {
        let rect = Rect::new(-30.0, -5.0, 40.0, 10.0);

        assert_eq!(PixelRect::covering(&rect, 1.0), pixels(0, 0, 10, 5));
    }

    #[test]
    fn rectangles_apart_intersect_in_nothing() {
        let intersection = pixels(0, 0, 10, 10).intersection(&pixels(20, 0, 10, 10));

        assert!(intersection.is_empty());
    }

    #[test]
    fn overlapping_rectangles_intersect_in_their_shared_pixels() {
        let intersection = pixels(0, 0, 10, 10).intersection(&pixels(4, 6, 10, 10));

        assert_eq!(intersection, pixels(4, 6, 6, 4));
    }
}
