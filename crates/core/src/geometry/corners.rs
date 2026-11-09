use std::ops::{Index, IndexMut};

use morok_style::Corner;

/// One value for each corner of a box.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Corners<T> {
    pub top_left: T,
    pub top_right: T,
    pub bottom_right: T,
    pub bottom_left: T,
}

impl<T> Index<Corner> for Corners<T> {
    type Output = T;

    fn index(&self, corner: Corner) -> &T {
        match corner {
            Corner::TopLeft => &self.top_left,
            Corner::TopRight => &self.top_right,
            Corner::BottomRight => &self.bottom_right,
            Corner::BottomLeft => &self.bottom_left,
        }
    }
}

impl<T> IndexMut<Corner> for Corners<T> {
    fn index_mut(&mut self, corner: Corner) -> &mut T {
        match corner {
            Corner::TopLeft => &mut self.top_left,
            Corner::TopRight => &mut self.top_right,
            Corner::BottomRight => &mut self.bottom_right,
            Corner::BottomLeft => &mut self.bottom_left,
        }
    }
}
