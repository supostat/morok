use crate::layout::track_breadth::TrackBreadth;
use crate::length_percentage::LengthPercentage;

#[derive(Clone, Debug, PartialEq)]
pub enum TrackSize {
    Breadth(TrackBreadth),
    MinMax { min: TrackBreadth, max: TrackBreadth },
    FitContent(LengthPercentage),
}
