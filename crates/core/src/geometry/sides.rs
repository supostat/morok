use std::ops::{Index, IndexMut};

use morok_style::Side;

/// One value for each side of a box.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Sides<T> {
    pub top: T,
    pub right: T,
    pub bottom: T,
    pub left: T,
}

impl<T: Copy> Sides<T> {
    pub const fn uniform(value: T) -> Self {
        Self {
            top: value,
            right: value,
            bottom: value,
            left: value,
        }
    }

    /// The same sides with `map` applied to each.
    pub fn map<U>(self, map: impl Fn(T) -> U) -> Sides<U> {
        Sides {
            top: map(self.top),
            right: map(self.right),
            bottom: map(self.bottom),
            left: map(self.left),
        }
    }
}

impl<T> Index<Side> for Sides<T> {
    type Output = T;

    fn index(&self, side: Side) -> &T {
        match side {
            Side::Top => &self.top,
            Side::Right => &self.right,
            Side::Bottom => &self.bottom,
            Side::Left => &self.left,
        }
    }
}

impl<T> IndexMut<Side> for Sides<T> {
    fn index_mut(&mut self, side: Side) -> &mut T {
        match side {
            Side::Top => &mut self.top,
            Side::Right => &mut self.right,
            Side::Bottom => &mut self.bottom,
            Side::Left => &mut self.left,
        }
    }
}
