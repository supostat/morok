use std::collections::HashMap;

use morok_platform::FrameImage;

use crate::anti_aliasing::is_anti_aliasing;
use crate::lab_color::LabColor;
use crate::pixel_class::PixelClass;

const ALPHA: usize = 3;
const OPAQUE: u16 = 255;
/// A frame is drawn on a transparent surface, so two pixels are as different as they look on
/// the backdrop that tells them apart best: white shows a difference of a dark translucent
/// pixel, black the difference of a light one.
const BACKDROPS: [u16; 2] = [OPAQUE, 0];

/// The color gate measured over two frames. The area is the smallest rectangle that holds
/// both; a position one of the frames lacks mismatches.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct ColorMeasure {
    pub(crate) width: usize,
    pub(crate) height: usize,
    /// Row by row from the top.
    pub(crate) classes: Vec<PixelClass>,
    pub(crate) mismatching_pixels: usize,
    /// The CIEDE2000 differences of the pixels that are not anti-aliasing, summed and divided
    /// by the area.
    pub(crate) mean_difference: f32,
}

impl ColorMeasure {
    pub(crate) fn area(&self) -> usize {
        self.width * self.height
    }
}

/// Measures how the candidate differs from the etalon. A pixel that is not anti-aliasing
/// mismatches when its difference is above `pixel_difference`.
pub(crate) fn measure(etalon: &FrameImage, candidate: &FrameImage, pixel_difference: f32) -> ColorMeasure {
    let width = etalon.width.max(candidate.width);
    let height = etalon.height.max(candidate.height);
    let frames_are_one_size = etalon.width == candidate.width && etalon.height == candidate.height;
    let mut differences = HashMap::new();
    let mut classes = Vec::with_capacity(width * height);
    let mut mismatching_pixels = 0;
    let mut difference_sum = 0.0_f64;
    for y in 0..height {
        for x in 0..width {
            let class = match (pixel_at(etalon, x, y), pixel_at(candidate, x, y)) {
                (Some(etalon_pixel), Some(candidate_pixel)) if etalon_pixel == candidate_pixel => PixelClass::Matching,
                (Some(etalon_pixel), Some(candidate_pixel)) => {
                    if frames_are_one_size && is_anti_aliasing(etalon, candidate, x, y) {
                        PixelClass::AntiAliasing
                    } else {
                        let difference = *differences
                            .entry((etalon_pixel, candidate_pixel))
                            .or_insert_with(|| difference(etalon_pixel, candidate_pixel));
                        difference_sum += f64::from(difference);
                        if difference > pixel_difference {
                            PixelClass::Mismatching
                        } else {
                            PixelClass::Matching
                        }
                    }
                }
                _ => PixelClass::Mismatching,
            };
            mismatching_pixels += usize::from(class == PixelClass::Mismatching);
            classes.push(class);
        }
    }
    ColorMeasure {
        width,
        height,
        classes,
        mismatching_pixels,
        mean_difference: (difference_sum / (width * height) as f64) as f32,
    }
}

/// The CIEDE2000 difference of two premultiplied pixels on the backdrop that shows it best.
pub(crate) fn difference(etalon: [u8; 4], candidate: [u8; 4]) -> f32 {
    BACKDROPS
        .into_iter()
        .map(|backdrop| {
            LabColor::from_srgb(over(etalon, backdrop)).difference(LabColor::from_srgb(over(candidate, backdrop)))
        })
        .fold(0.0, f64::max) as f32
}

/// The opaque color of a premultiplied pixel laid over a backdrop of the given gray.
fn over(pixel: [u8; 4], backdrop: u16) -> [u8; 3] {
    let uncovered = (OPAQUE - u16::from(pixel[ALPHA])) * backdrop / OPAQUE;
    [0, 1, 2].map(|channel| (u16::from(pixel[channel]) + uncovered).min(OPAQUE) as u8)
}

fn pixel_at(image: &FrameImage, x: usize, y: usize) -> Option<[u8; 4]> {
    (x < image.width && y < image.height).then(|| image.pixel(x, y))
}
