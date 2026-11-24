use morok_core::accesskit::Orientation;
use morok_core::{Key, Point};
use morok_style::{Class, Declaration};

use crate::length_in_pixels::pixels;
use crate::styles::CLASSES;

/// The way a split view lays its panes out.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SplitAxis {
    /// Side by side, with upright dividers between them.
    #[default]
    Horizontal,
    /// One above another, with level dividers between them.
    Vertical,
}

impl SplitAxis {
    /// The class the view and the parts of its dividers take for the axis.
    pub(crate) fn class(self) -> Class {
        match self {
            Self::Horizontal => CLASSES.horizontal,
            Self::Vertical => CLASSES.vertical,
        }
    }

    /// Which way the line of a divider lies: across the axis.
    pub(crate) fn divider_orientation(self) -> Orientation {
        match self {
            Self::Horizontal => Orientation::Vertical,
            Self::Vertical => Orientation::Horizontal,
        }
    }

    /// How far along the axis a point of the surface lies.
    pub(crate) fn along(self, point: Point) -> f32 {
        match self {
            Self::Horizontal => point.x,
            Self::Vertical => point.y,
        }
    }

    /// The declaration that makes a pane `length` CSS pixels long along the axis.
    pub(crate) fn length(self, length: f32) -> Declaration {
        match self {
            Self::Horizontal => Declaration::Width(pixels(length)),
            Self::Vertical => Declaration::Height(pixels(length)),
        }
    }

    /// The arrow key that points along the axis: right, or down.
    pub(crate) fn forward_arrow(self) -> Key {
        match self {
            Self::Horizontal => Key::ArrowRight,
            Self::Vertical => Key::ArrowDown,
        }
    }

    /// The arrow key that points back along the axis: left, or up.
    pub(crate) fn backward_arrow(self) -> Key {
        match self {
            Self::Horizontal => Key::ArrowLeft,
            Self::Vertical => Key::ArrowUp,
        }
    }
}
