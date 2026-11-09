use morok_core::Rect;
use morok_platform::FrameImage;

use crate::comparison::Comparison;
use crate::mismatch::Mismatch;
use crate::pixel_class::PixelClass;
use crate::rendering::{DEVICE_SCALE, Rendering};

const CHANNELS: usize = 4;
const ALPHA: usize = 3;
const WHITE: u32 = 255;
/// The etalon, the rendering and the marks, side by side.
const PANELS: usize = 3;
const MARKS_PANEL: usize = 2;
/// The share of each channel in the brightness of a color, in thousandths.
const LUMA_WEIGHTS: [u32; 3] = [299, 587, 114];
/// How much of its contrast the etalon keeps under the marks, in percent.
const ETALON_CONTRAST: u32 = 30;
const MISMATCHING_PIXEL: [u8; 3] = [255, 0, 0];
const ANTI_ALIASING_PIXEL: [u8; 3] = [255, 196, 0];
const ETALON_BOX: [u8; 3] = [0, 150, 70];
const CANDIDATE_BOX: [u8; 3] = [200, 0, 220];

/// A picture of what a comparison found, three panels wide: the etalon, the rendering, and
/// the marks. Both frames are laid over white. The marks are drawn on the etalon faded to
/// gray: the mismatching pixels in red, the pixels left out as anti-aliasing in yellow, and
/// for every box that lies off its etalon the outline of the etalon in green and of the
/// rendering in magenta.
pub(crate) fn of(etalon: &Rendering, candidate: &Rendering, comparison: &Comparison) -> FrameImage {
    let measure = &comparison.color_measure;
    let mut diff = FrameImage {
        width: measure.width * PANELS,
        height: measure.height,
        pixels: Vec::with_capacity(measure.area() * PANELS * CHANNELS),
    };
    for y in 0..measure.height {
        for frame in [&etalon.image, &candidate.image] {
            for x in 0..measure.width {
                diff.pixels.extend(over_white(frame, x, y));
                diff.pixels.push(WHITE as u8);
            }
        }
        for x in 0..measure.width {
            let color = match measure.classes[y * measure.width + x] {
                PixelClass::Matching => faded(over_white(&etalon.image, x, y)),
                PixelClass::AntiAliasing => ANTI_ALIASING_PIXEL,
                PixelClass::Mismatching => MISMATCHING_PIXEL,
            };
            diff.pixels.extend(color);
            diff.pixels.push(WHITE as u8);
        }
    }
    for mismatch in &comparison.mismatches {
        if let Mismatch::ElementBox { etalon, candidate } = mismatch {
            outline(&mut diff, measure.width, etalon.rect(), ETALON_BOX);
            outline(&mut diff, measure.width, candidate.rect(), CANDIDATE_BOX);
        }
    }
    diff
}

/// The color of the pixel laid over white; white where the frame has no pixel.
fn over_white(frame: &FrameImage, x: usize, y: usize) -> [u8; 3] {
    if x >= frame.width || y >= frame.height {
        return [WHITE as u8; 3];
    }
    let pixel = frame.pixel(x, y);
    let uncovered = WHITE - u32::from(pixel[ALPHA]);
    [0, 1, 2].map(|channel| (u32::from(pixel[channel]) + uncovered).min(WHITE) as u8)
}

/// The color as a light gray that leaves the marks standing out.
fn faded(color: [u8; 3]) -> [u8; 3] {
    let brightness = LUMA_WEIGHTS
        .into_iter()
        .zip(color)
        .map(|(weight, channel)| weight * u32::from(channel))
        .sum::<u32>()
        / 1000;
    [(WHITE - (WHITE - brightness) * ETALON_CONTRAST / 100) as u8; 3]
}

/// Draws the edges of a box given in CSS pixels on the marks panel, one device pixel wide.
fn outline(diff: &mut FrameImage, panel_width: usize, css_box: Rect, color: [u8; 3]) {
    if panel_width == 0 || diff.height == 0 {
        return;
    }
    let device_pixel =
        |css_pixels: f32, limit: usize| ((css_pixels * DEVICE_SCALE).round().max(0.0) as usize).min(limit - 1);
    let left = device_pixel(css_box.x, panel_width);
    let right = device_pixel(css_box.right(), panel_width);
    let top = device_pixel(css_box.y, diff.height);
    let bottom = device_pixel(css_box.bottom(), diff.height);
    let mut paint = |x: usize, y: usize| {
        let start = (y * diff.width + MARKS_PANEL * panel_width + x) * CHANNELS;
        diff.pixels[start..start + ALPHA].copy_from_slice(&color);
    };
    for x in left..=right {
        paint(x, top);
        paint(x, bottom);
    }
    for y in top..=bottom {
        paint(left, y);
        paint(right, y);
    }
}
