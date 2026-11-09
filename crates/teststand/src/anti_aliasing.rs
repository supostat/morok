use morok_platform::FrameImage;

const ALPHA: usize = 3;
const OPAQUE: i32 = 255;
/// The share of each channel in the brightness of a color, in thousandths.
const LUMA_WEIGHTS: [i32; 3] = [299, 587, 114];
/// A neighbour as bright as the pixel counts as its sibling, and so does every position the
/// edge of the image leaves out; a pixel with more siblings than this lies in a flat area.
const SIBLINGS_OF_AN_EDGE: usize = 2;
/// A frame is drawn on a transparent surface, so an edge shows on a white or on a black
/// backdrop: a white box has no edge over white, a black text none over black.
const BACKDROPS: [i32; 2] = [OPAQUE, 0];

/// Whether the pixel smooths an edge in either image: two renderers draw the same edge with
/// slightly different coverage, which is no difference in what they draw.
///
/// The detector is the one of Vyšniauskas, "Anti-aliased Pixel and Intensity Slope Detector":
/// a pixel on an edge has both a darker and a brighter neighbour and few neighbours as bright
/// as itself, and the darkest or the brightest of them lies in an area that is flat in both
/// images.
///
/// # Panics
///
/// When the images differ in size.
pub(crate) fn is_anti_aliasing(etalon: &FrameImage, candidate: &FrameImage, x: usize, y: usize) -> bool {
    assert!(
        etalon.width == candidate.width && etalon.height == candidate.height,
        "anti-aliasing is told on images of one size"
    );
    BACKDROPS.into_iter().any(|backdrop| {
        smooths_an_edge(etalon, candidate, x, y, backdrop) || smooths_an_edge(candidate, etalon, x, y, backdrop)
    })
}

fn smooths_an_edge(image: &FrameImage, other: &FrameImage, x: usize, y: usize, backdrop: i32) -> bool {
    let brightness = brightness_over(image.pixel(x, y), backdrop);
    let mut siblings = positions_outside(image, x, y);
    let mut darkest: Option<(i32, usize, usize)> = None;
    let mut brightest: Option<(i32, usize, usize)> = None;
    for (neighbour_x, neighbour_y) in neighbours(image, x, y) {
        let delta = brightness_over(image.pixel(neighbour_x, neighbour_y), backdrop) - brightness;
        if delta == 0 {
            siblings += 1;
            if siblings > SIBLINGS_OF_AN_EDGE {
                return false;
            }
        } else if delta < darkest.map_or(0, |(darkest_delta, ..)| darkest_delta) {
            darkest = Some((delta, neighbour_x, neighbour_y));
        } else if delta > brightest.map_or(0, |(brightest_delta, ..)| brightest_delta) {
            brightest = Some((delta, neighbour_x, neighbour_y));
        }
    }
    let (Some((_, darkest_x, darkest_y)), Some((_, brightest_x, brightest_y))) = (darkest, brightest) else {
        return false;
    };
    let is_flat_in_both = |x: usize, y: usize| lies_in_a_flat_area(image, x, y) && lies_in_a_flat_area(other, x, y);
    is_flat_in_both(darkest_x, darkest_y) || is_flat_in_both(brightest_x, brightest_y)
}

/// Whether more neighbours than an edge has hold exactly the color of the pixel.
fn lies_in_a_flat_area(image: &FrameImage, x: usize, y: usize) -> bool {
    let pixel = image.pixel(x, y);
    let equal_neighbours = neighbours(image, x, y)
        .filter(|(neighbour_x, neighbour_y)| image.pixel(*neighbour_x, *neighbour_y) == pixel)
        .count();
    positions_outside(image, x, y) + equal_neighbours > SIBLINGS_OF_AN_EDGE
}

/// One when the pixel lies on the edge of the image, where some of its eight neighbours are
/// missing.
fn positions_outside(image: &FrameImage, x: usize, y: usize) -> usize {
    usize::from(x == 0 || y == 0 || x + 1 == image.width || y + 1 == image.height)
}

/// The positions around the pixel that lie inside the image.
fn neighbours(image: &FrameImage, x: usize, y: usize) -> impl Iterator<Item = (usize, usize)> {
    let columns = x.saturating_sub(1)..=(x + 1).min(image.width - 1);
    let rows = y.saturating_sub(1)..=(y + 1).min(image.height - 1);
    rows.flat_map(move |row| columns.clone().map(move |column| (column, row)))
        .filter(move |position| *position != (x, y))
}

/// The brightness of a premultiplied pixel laid over an opaque backdrop of the given gray.
fn brightness_over(pixel: [u8; 4], backdrop: i32) -> i32 {
    let uncovered = (OPAQUE - i32::from(pixel[ALPHA])) * backdrop / OPAQUE;
    LUMA_WEIGHTS
        .into_iter()
        .zip(pixel)
        .map(|(weight, channel)| weight * (i32::from(channel) + uncovered))
        .sum()
}
