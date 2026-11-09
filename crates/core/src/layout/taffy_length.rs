use taffy::{Dimension, LengthPercentage, LengthPercentageAuto, MaxTrackSizingFunction, MinTrackSizingFunction};

use crate::layout::calc_handle::CalcHandle;
use crate::style::resolved_length::ResolvedLength;

/// A Taffy value that holds a length, a percentage, or a handle to a mix of the two.
pub(crate) trait TaffyLength: Sized {
    fn pixels(pixels: f32) -> Self;
    fn fraction(fraction: f32) -> Self;
    fn mixed(handle: CalcHandle) -> Self;

    /// `place` names the length in case Taffy has to ask for its value against a basis.
    fn of(length: ResolvedLength, place: CalcHandle) -> Self {
        match length.fraction {
            None => Self::pixels(length.pixels),
            Some(fraction) if length.pixels == 0.0 => Self::fraction(fraction),
            Some(_) => Self::mixed(place),
        }
    }
}

impl TaffyLength for LengthPercentage {
    fn pixels(pixels: f32) -> Self {
        Self::length(pixels)
    }

    fn fraction(fraction: f32) -> Self {
        Self::percent(fraction)
    }

    fn mixed(handle: CalcHandle) -> Self {
        Self::calc(handle.to_pointer())
    }
}

impl TaffyLength for LengthPercentageAuto {
    fn pixels(pixels: f32) -> Self {
        Self::length(pixels)
    }

    fn fraction(fraction: f32) -> Self {
        Self::percent(fraction)
    }

    fn mixed(handle: CalcHandle) -> Self {
        Self::calc(handle.to_pointer())
    }
}

impl TaffyLength for Dimension {
    fn pixels(pixels: f32) -> Self {
        Self::length(pixels)
    }

    fn fraction(fraction: f32) -> Self {
        Self::percent(fraction)
    }

    fn mixed(handle: CalcHandle) -> Self {
        Self::calc(handle.to_pointer())
    }
}

impl TaffyLength for MinTrackSizingFunction {
    fn pixels(pixels: f32) -> Self {
        Self::length(pixels)
    }

    fn fraction(fraction: f32) -> Self {
        Self::percent(fraction)
    }

    fn mixed(handle: CalcHandle) -> Self {
        Self::calc(handle.to_pointer())
    }
}

impl TaffyLength for MaxTrackSizingFunction {
    fn pixels(pixels: f32) -> Self {
        Self::length(pixels)
    }

    fn fraction(fraction: f32) -> Self {
        Self::percent(fraction)
    }

    fn mixed(handle: CalcHandle) -> Self {
        Self::calc(handle.to_pointer())
    }
}
