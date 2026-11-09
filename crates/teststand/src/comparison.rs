use crate::color_gate::{self, ColorMeasure};
use crate::geometry_gate;
use crate::mismatch::Mismatch;
use crate::rendering::Rendering;
use crate::thresholds::Thresholds;

/// A rendering held against its etalon under one color scheme.
#[derive(Clone, Debug, PartialEq)]
pub struct Comparison {
    /// Every way the rendering fails the gates; none when it matches.
    pub mismatches: Vec<Mismatch>,
    pub(crate) color_measure: ColorMeasure,
}

impl Comparison {
    pub fn matches(&self) -> bool {
        self.mismatches.is_empty()
    }

    /// How many pixels that are not anti-aliasing differ by more than a pixel may.
    pub fn mismatching_pixels(&self) -> usize {
        self.color_measure.mismatching_pixels
    }

    /// The mean CIEDE2000 difference over the case, anti-aliasing left out.
    pub fn mean_difference(&self) -> f32 {
        self.color_measure.mean_difference
    }
}

/// Holds the candidate against the etalon. The geometry gate: every element box within the
/// tolerance of the etalon's. The color gate, on pixels that are not anti-aliasing: no more
/// than the allowed share of the case mismatches, and the mean difference over the case
/// stays under its limit.
pub fn compare(etalon: &Rendering, candidate: &Rendering, thresholds: &Thresholds) -> Comparison {
    let mut mismatches = geometry_gate::mismatches(&etalon.boxes, &candidate.boxes, thresholds.box_tolerance);
    let etalon_size = (etalon.image.width, etalon.image.height);
    let candidate_size = (candidate.image.width, candidate.image.height);
    if etalon_size != candidate_size {
        mismatches.push(Mismatch::ImageSize {
            etalon: etalon_size,
            candidate: candidate_size,
        });
    }
    let color_measure = color_gate::measure(&etalon.image, &candidate.image, thresholds.pixel_difference);
    let area = color_measure.area();
    if color_measure.mismatching_pixels as f32 > thresholds.mismatch_fraction * area as f32 {
        mismatches.push(Mismatch::MismatchingPixels {
            pixels: color_measure.mismatching_pixels,
            area,
        });
    }
    if color_measure.mean_difference > thresholds.mean_difference {
        mismatches.push(Mismatch::MeanDifference {
            mean: color_measure.mean_difference,
        });
    }
    Comparison {
        mismatches,
        color_measure,
    }
}
