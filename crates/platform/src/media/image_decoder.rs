use std::ffi::c_void;

use objc2_core_foundation::{CFData, CGPoint, CGRect, CGSize};
use objc2_core_graphics::{
    CGBitmapContextCreate, CGColorSpace, CGContext, CGImage, CGImageAlphaInfo, CGImageByteOrderInfo, kCGColorSpaceSRGB,
};
use objc2_image_io::CGImageSource;

use crate::media::decoded_image::DecodedImage;

const BITS_PER_CHANNEL: usize = 8;
const BYTES_PER_PIXEL: usize = 4;

/// Decodes an encoded image — PNG, JPEG, HEIC or any other format the system reads — into
/// premultiplied pixels in sRGB. Returns none for data the system does not read as an image.
pub(crate) fn decode(encoded: &[u8]) -> Option<DecodedImage> {
    let data = CFData::from_bytes(encoded);
    // SAFETY: no options are passed.
    let source = unsafe { CGImageSource::with_data(&data, None) }?;
    // SAFETY: no options are passed; the first image of the source is read.
    let image = unsafe { source.image_at_index(0, None) }?;
    let width = CGImage::width(Some(&image));
    let height = CGImage::height(Some(&image));
    if width == 0 || height == 0 {
        return None;
    }
    let mut pixels = vec![0_u8; width * height * BYTES_PER_PIXEL];
    // SAFETY: the name is a constant of Core Graphics.
    let color_space = CGColorSpace::with_name(Some(unsafe { kCGColorSpaceSRGB }));
    let layout = CGImageAlphaInfo::PremultipliedFirst.0 | CGImageByteOrderInfo::Order32Little.0;
    // SAFETY: the buffer holds `height` rows of `width` pixels in the layout named.
    let context = unsafe {
        CGBitmapContextCreate(
            pixels.as_mut_ptr().cast::<c_void>(),
            width,
            height,
            BITS_PER_CHANNEL,
            width * BYTES_PER_PIXEL,
            color_space.as_deref(),
            layout,
        )
    }?;
    CGContext::draw_image(
        Some(&context),
        CGRect::new(CGPoint::new(0.0, 0.0), CGSize::new(width as f64, height as f64)),
        Some(&image),
    );
    drop(context);
    Some(DecodedImage {
        width: width as u32,
        height: height as u32,
        pixels,
    })
}
