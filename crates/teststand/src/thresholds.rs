use std::path::Path;

use serde::Deserialize;

use crate::stand_error::StandError;
use crate::toml_file;

/// How far a rendering of the kit may be from its etalon and still match it.
#[derive(Clone, Copy, Debug, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Thresholds {
    /// The distance, in CSS pixels, an edge of an element box may be from the same edge in the etalon.
    pub box_tolerance: f32,
    /// The CIEDE2000 difference above which a pixel that is not anti-aliasing mismatches.
    pub pixel_difference: f32,
    /// The share of the pixels of a case that may mismatch.
    pub mismatch_fraction: f32,
    /// The mean CIEDE2000 difference over a case that may not be exceeded.
    pub mean_difference: f32,
}

impl Thresholds {
    pub fn read(path: &Path) -> Result<Self, StandError> {
        toml_file::read(path)
    }
}
