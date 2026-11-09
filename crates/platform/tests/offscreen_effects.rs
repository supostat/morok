mod support;

use std::sync::LazyLock;

use morok_core::Size;
use morok_platform::{FrameImage, Renderer};
use support::Scene;

const SURFACE: Size = Size {
    width: 200.0,
    height: 120.0,
};
const CLEAR: [u8; 4] = [0, 0, 0, 0];
const RED: [u8; 4] = [255, 0, 0, 255];
const BLUE: [u8; 4] = [0, 0, 255, 255];
const WHITE: [u8; 4] = [255, 255, 255, 255];
/// The largest difference of a channel that rounding to 8 bits and the arithmetic of the GPU
/// explain.
const CHANNEL_TOLERANCE: i32 = 2;

fn renderer() -> &'static Renderer {
    static RENDERER: LazyLock<Renderer> = LazyLock::new(|| Renderer::new().expect("this Mac runs Metal 4"));
    &RENDERER
}

/// Draws a scene whose root holds one element per class list of `elements`, in that order.
fn render(rules: &str, elements: &[&[&str]]) -> FrameImage {
    let mut scene = Scene::new(rules, SURFACE);
    for classes in elements {
        scene.add(scene.root(), classes);
    }
    scene.render(renderer(), 1.0)
}

fn assert_pixel_near(image: &FrameImage, x: usize, y: usize, expected: [u8; 4]) {
    let actual = image.pixel(x, y);
    let is_near = actual
        .iter()
        .zip(expected)
        .all(|(channel, expected)| (i32::from(*channel) - i32::from(expected)).abs() <= CHANNEL_TOLERANCE);
    assert!(
        is_near,
        "the pixel ({x}, {y}) is {actual:?}, expected about {expected:?}"
    );
}

#[test]
fn a_translated_box_is_drawn_where_the_transform_moves_it() {
    let image = render(
        ".box { position: absolute; left: 20px; top: 10px; width: 50px; height: 40px; \
         background-color: var(--color-red); transform: translate(100px, 50px); }",
        &[&["box"]],
    );

    assert_eq!(image.pixel(40, 30), CLEAR);
    assert_eq!(image.pixel(140, 80), RED);
    assert_eq!(image.pixel(119, 80), CLEAR);
}

#[test]
fn a_scaled_box_grows_about_its_origin() {
    let image = render(
        ".box { position: absolute; left: 50px; top: 30px; width: 40px; height: 40px; \
         background-color: var(--color-red); transform: scale(2); }",
        &[&["box"]],
    );

    assert_eq!(image.pixel(70, 50), RED);
    assert_eq!(image.pixel(35, 15), RED);
    assert_eq!(image.pixel(105, 85), RED);
    assert_eq!(image.pixel(25, 50), CLEAR);
    assert_eq!(image.pixel(115, 50), CLEAR);
}

#[test]
fn a_rotated_box_covers_its_diagonal_and_leaves_its_old_corners() {
    let image = render(
        ".box { position: absolute; left: 60px; top: 20px; width: 80px; height: 80px; \
         background-color: var(--color-red); transform: rotate(45deg); }",
        &[&["box"]],
    );

    assert_eq!(image.pixel(100, 60), RED);
    assert_eq!(image.pixel(100, 10), RED);
    assert_eq!(image.pixel(65, 25), CLEAR);
    assert_eq!(image.pixel(135, 95), CLEAR);
}

#[test]
fn group_opacity_fades_a_box_and_what_it_covers_as_one() {
    let image = render(
        ".group { position: absolute; left: 20px; top: 10px; width: 100px; height: 60px; \
         background-color: var(--color-red); opacity: 0.5; }\n\
         .inner { position: absolute; left: 20px; top: 10px; width: 40px; height: 40px; background-color: var(--color-blue); }",
        &[&["group"]],
    );

    assert_pixel_near(&image, 100, 40, [128, 0, 0, 128]);
}

#[test]
fn a_child_inside_a_faded_group_does_not_show_the_group_through_it() {
    let mut scene = Scene::new(
        ".group { position: absolute; left: 20px; top: 10px; width: 100px; height: 60px; \
         background-color: var(--color-red); opacity: 0.5; }\n\
         .inner { position: absolute; left: 10px; top: 10px; width: 40px; height: 40px; background-color: var(--color-blue); }",
        SURFACE,
    );
    let group = scene.add(scene.root(), &["group"]);
    scene.add(group, &["inner"]);

    let image = scene.render(renderer(), 1.0);

    assert_pixel_near(&image, 50, 40, [0, 0, 128, 128]);
    assert_pixel_near(&image, 100, 40, [128, 0, 0, 128]);
}

#[test]
fn a_grayscale_filter_takes_the_color_out_of_a_box() {
    let image = render(
        ".box { position: absolute; left: 20px; top: 10px; width: 100px; height: 60px; \
         background-color: var(--color-red); filter: grayscale(1); }",
        &[&["box"]],
    );

    let gray = (0.2126 * 255.0_f32).round() as u8;
    assert_pixel_near(&image, 70, 40, [gray, gray, gray, 255]);
}

#[test]
fn a_blur_filter_softens_the_edge_of_a_box_and_reaches_past_it() {
    let image = render(
        ".box { position: absolute; left: 50px; top: 30px; width: 100px; height: 60px; \
         background-color: var(--color-red); filter: blur(4px); }",
        &[&["box"]],
    );

    assert_pixel_near(&image, 100, 60, RED);
    let on_edge = image.pixel(50, 60);
    assert!(
        on_edge[3] > 100 && on_edge[3] < 160,
        "the edge is half covered: {on_edge:?}"
    );
    assert!(image.pixel(44, 60)[3] > 0, "the blur reaches past the box");
    assert_eq!(image.pixel(30, 60), CLEAR);
}

#[test]
fn multiply_blends_a_box_with_what_lies_under_it() {
    let image = render(
        ".under { position: absolute; left: 0; top: 0; width: 200px; height: 120px; background-color: var(--color-paper); }\n\
         .over { position: absolute; left: 20px; top: 10px; width: 100px; height: 60px; \
         background-color: var(--color-red); mix-blend-mode: multiply; }",
        &[&["under"], &["over"]],
    );

    assert_pixel_near(&image, 70, 40, RED);
    assert_pixel_near(&image, 160, 40, WHITE);
}

#[test]
fn screen_lightens_a_box_against_a_dark_backdrop() {
    let image = render(
        ".under { position: absolute; left: 0; top: 0; width: 200px; height: 120px; background-color: var(--color-blue); }\n\
         .over { position: absolute; left: 20px; top: 10px; width: 100px; height: 60px; \
         background-color: var(--color-red); mix-blend-mode: screen; }",
        &[&["under"], &["over"]],
    );

    assert_pixel_near(&image, 70, 40, [255, 0, 255, 255]);
    assert_pixel_near(&image, 160, 40, BLUE);
}

#[test]
fn a_backdrop_blur_softens_what_lies_under_a_box_inside_its_frame() {
    let image = render(
        ".left { position: absolute; left: 0; top: 0; width: 100px; height: 120px; background-color: var(--color-red); }\n\
         .right { position: absolute; left: 100px; top: 0; width: 100px; height: 120px; background-color: var(--color-blue); }\n\
         .glass { position: absolute; left: 60px; top: 20px; width: 80px; height: 80px; backdrop-filter: blur(6px); }",
        &[&["left"], &["right"], &["glass"]],
    );

    let blended = image.pixel(100, 60);
    assert!(
        blended[0] > 60 && blended[0] < 200 && blended[2] > 60 && blended[2] < 200,
        "the seam under the glass is blurred: {blended:?}"
    );
    assert_eq!(image.pixel(100, 10), BLUE);
    assert_eq!(image.pixel(99, 10), RED);
}

#[test]
fn an_opacity_filter_fades_a_box_like_group_opacity_does() {
    let image = render(
        ".box { position: absolute; left: 20px; top: 10px; width: 100px; height: 60px; \
         background-color: var(--color-red); filter: opacity(40%); }",
        &[&["box"]],
    );

    assert_pixel_near(&image, 70, 40, [102, 0, 0, 102]);
}
