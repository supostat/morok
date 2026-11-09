use std::io::Cursor;

use morok_platform::FrameImage;
use png::{BitDepth, ColorType, Decoder, Encoder};

const CHANNELS: usize = 4;
const ALPHA: usize = 3;
const OPAQUE: u32 = 255;

/// Reads a PNG of eight bits per channel into a frame. Chrome leaves the alpha channel out of
/// a screenshot without a transparent pixel, so both RGB and RGBA are read. A PNG holds colors
/// that are not premultiplied by alpha and a frame holds premultiplied ones, so the colors are
/// multiplied on the way in.
pub(crate) fn decode(png: &[u8]) -> Result<FrameImage, String> {
    let mut reader = Decoder::new(Cursor::new(png))
        .read_info()
        .map_err(|error| error.to_string())?;
    let buffer_size = reader
        .output_buffer_size()
        .ok_or_else(|| String::from("the image is too large to hold in memory"))?;
    let mut decoded = vec![0; buffer_size];
    let frame = reader.next_frame(&mut decoded).map_err(|error| error.to_string())?;
    decoded.truncate(frame.buffer_size());
    let pixels = match (frame.color_type, frame.bit_depth) {
        (ColorType::Rgba, BitDepth::Eight) => premultiplied(decoded),
        (ColorType::Rgb, BitDepth::Eight) => decoded
            .chunks_exact(ALPHA)
            .flat_map(|color| [color[0], color[1], color[2], OPAQUE as u8])
            .collect(),
        (color_type, bit_depth) => {
            return Err(format!(
                "expected eight-bit RGB or RGBA, found {color_type:?} of {bit_depth:?} bits"
            ));
        }
    };
    Ok(FrameImage {
        width: frame.width as usize,
        height: frame.height as usize,
        pixels,
    })
}

fn premultiplied(mut pixels: Vec<u8>) -> Vec<u8> {
    for pixel in pixels.chunks_exact_mut(CHANNELS) {
        let alpha = u32::from(pixel[ALPHA]);
        for channel in &mut pixel[..ALPHA] {
            *channel = ((u32::from(*channel) * alpha + OPAQUE / 2) / OPAQUE) as u8;
        }
    }
    pixels
}

/// Writes a frame as a PNG of eight-bit RGBA. Decoding the result gives the frame back: a
/// premultiplied color survives the division by its alpha and the multiplication that undoes it.
pub(crate) fn encode(image: &FrameImage) -> Vec<u8> {
    let mut straight = image.pixels.clone();
    for pixel in straight.chunks_exact_mut(CHANNELS) {
        let alpha = u32::from(pixel[ALPHA]);
        if alpha == 0 {
            continue;
        }
        for channel in &mut pixel[..ALPHA] {
            *channel = ((u32::from(*channel) * OPAQUE + alpha / 2) / alpha).min(OPAQUE) as u8;
        }
    }
    let mut png = Vec::new();
    let mut encoder = Encoder::new(&mut png, image.width as u32, image.height as u32);
    encoder.set_color(ColorType::Rgba);
    encoder.set_depth(BitDepth::Eight);
    let mut writer = encoder.write_header().expect("a vector takes the header of a PNG");
    writer
        .write_image_data(&straight)
        .expect("the frame holds a pixel for every position of its size");
    writer.finish().expect("a vector takes the end of a PNG");
    png
}

#[cfg(test)]
mod tests {
    use morok_platform::FrameImage;

    use super::{decode, encode};

    #[test]
    fn a_frame_with_translucent_pixels_comes_back_as_it_was() {
        let frame = FrameImage {
            width: 3,
            height: 2,
            pixels: vec![
                0, 0, 0, 0, 255, 255, 255, 255, 20, 143, 120, 255, 1, 0, 1, 1, 10, 71, 60, 128, 17, 19, 24, 36,
            ],
        };

        assert_eq!(decode(&encode(&frame)), Ok(frame));
    }

    #[test]
    fn a_screenshot_without_an_alpha_channel_is_read_as_opaque() {
        let mut png = Vec::new();
        let mut encoder = png::Encoder::new(&mut png, 2, 1);
        encoder.set_color(png::ColorType::Rgb);
        encoder.set_depth(png::BitDepth::Eight);
        let mut writer = encoder.write_header().expect("a vector takes the header of a PNG");
        writer
            .write_image_data(&[20, 143, 120, 0, 0, 0])
            .expect("two pixels of three channels");
        writer.finish().expect("a vector takes the end of a PNG");

        let frame = decode(&png).expect("an RGB image is read");

        assert_eq!(frame.pixels, [20, 143, 120, 255, 0, 0, 0, 255]);
    }

    #[test]
    fn what_is_no_png_is_refused() {
        assert!(decode(b"not a png").is_err());
    }
}
