mod support;

use std::sync::LazyLock;
use std::thread;
use std::time::{Duration, Instant};

use morok_core::{CanvasId, ElementId, ImageId, Rect, Role, Size};
use morok_platform::{
    CanvasEncoder, CanvasPainter, FrameImage, ImageStatus, OffscreenTarget, Renderer, RendererOptions,
};
use morok_style::Color;
use support::Scene;

const SKY_PNG: &[u8] = include_bytes!("images/sky.png");
const SKY_JPEG: &[u8] = include_bytes!("images/sky.jpg");
const SKY_HEIC: &[u8] = include_bytes!("images/sky.heic");
const FIELD_PNG: &[u8] = include_bytes!("images/field.png");
const SKY_SIZE: Size = Size {
    width: 64.0,
    height: 48.0,
};
/// The sky at its middle, as the file encodes it.
const SKY_MIDDLE: [u8; 4] = [100, 141, 159, 255];
const CLEAR: [u8; 4] = [0, 0, 0, 0];
/// Decoders of lossy formats and the GPU's filtering may move a channel this far.
const CHANNEL_TOLERANCE: i32 = 8;
const DECODING_TIMEOUT: Duration = Duration::from_secs(5);
/// The bytes of one `field.png` texture: 512 by 384 texels of four bytes.
const FIELD_BYTES: usize = 512 * 384 * 4;

fn renderer() -> &'static Renderer {
    static RENDERER: LazyLock<Renderer> = LazyLock::new(|| Renderer::new().expect("this Mac runs Metal 4"));
    &RENDERER
}

/// Hands the image in and waits for the decoder on its thread.
fn decoded(renderer: &Renderer, encoded: &[u8]) -> (ImageId, Size) {
    let image = renderer.add_image(encoded.to_vec());
    let started = Instant::now();
    loop {
        match renderer.image_status(image) {
            ImageStatus::Decoded(size) => return (image, size),
            ImageStatus::Unreadable => panic!("the image is unreadable"),
            ImageStatus::Decoding => {
                assert!(started.elapsed() < DECODING_TIMEOUT, "the image decodes in time");
                thread::sleep(Duration::from_millis(5));
            }
        }
    }
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

/// A scene of one image element styled by `rules` on a 200 by 120 surface.
fn render_image(renderer: &Renderer, rules: &str, image: ImageId, size: Size) -> FrameImage {
    let mut scene = Scene::new(
        rules,
        Size {
            width: 200.0,
            height: 120.0,
        },
    );
    let element = scene.tree.create(Role::Image);
    scene.add_classes(element, &["picture"]);
    let root = scene.root();
    scene.tree.append_child(root, element);
    scene.tree.set_image(element, image, Some(size));
    scene.render(renderer, 1.0)
}

#[test]
fn png_jpeg_and_heic_decode_on_a_thread_of_their_own_to_the_same_size() {
    let renderer = renderer();
    for encoded in [SKY_PNG, SKY_JPEG, SKY_HEIC] {
        let image = renderer.add_image(encoded.to_vec());
        let handed_in = renderer.image_status(image);
        assert!(
            matches!(handed_in, ImageStatus::Decoding | ImageStatus::Decoded(_)),
            "the image is taken without waiting: {handed_in:?}"
        );
        let (_, size) = decoded(renderer, encoded);
        assert_eq!(size, SKY_SIZE);
    }
}

#[test]
fn data_that_is_no_image_is_reported_unreadable() {
    let renderer = renderer();
    let image = renderer.add_image(b"not an image".to_vec());
    let started = Instant::now();
    while renderer.image_status(image) == ImageStatus::Decoding {
        assert!(started.elapsed() < DECODING_TIMEOUT, "the decoder gives up in time");
        thread::sleep(Duration::from_millis(5));
    }
    assert_eq!(renderer.image_status(image), ImageStatus::Unreadable);
}

#[test]
fn an_image_filling_a_box_of_its_own_size_shows_its_pixels() {
    let renderer = renderer();
    let (image, size) = decoded(renderer, SKY_PNG);
    let frame = render_image(
        renderer,
        ".picture { position: absolute; left: 20px; top: 10px; width: 64px; height: 48px; }",
        image,
        size,
    );

    assert_pixel_near(&frame, 52, 34, SKY_MIDDLE);
    assert_eq!(frame.pixel(10, 10), CLEAR);
}

#[test]
fn contain_leaves_the_sides_of_a_wider_box_clear_and_cover_fills_them() {
    let renderer = renderer();
    let (image, size) = decoded(renderer, SKY_JPEG);
    let contained = render_image(
        renderer,
        ".picture { position: absolute; left: 20px; top: 10px; width: 128px; height: 48px; object-fit: contain; }",
        image,
        size,
    );
    let covered = render_image(
        renderer,
        ".picture { position: absolute; left: 20px; top: 10px; width: 128px; height: 48px; object-fit: cover; }",
        image,
        size,
    );

    assert_eq!(contained.pixel(30, 34), CLEAR);
    assert_pixel_near(&contained, 84, 34, SKY_MIDDLE);
    assert_eq!(contained.pixel(140, 34), CLEAR);
    assert_ne!(covered.pixel(30, 34), CLEAR);
    assert_ne!(covered.pixel(140, 34), CLEAR);
}

#[test]
fn an_image_without_pixels_yet_shows_its_box_alone() {
    let renderer = renderer();
    let image = renderer.add_image(FIELD_PNG.to_vec());
    let mut scene = Scene::new(
        ".picture { position: absolute; left: 20px; top: 10px; width: 64px; height: 48px; \
         background-color: var(--color-red); }",
        Size {
            width: 200.0,
            height: 120.0,
        },
    );
    let element = scene.tree.create(Role::Image);
    scene.add_classes(element, &["picture"]);
    let root = scene.root();
    scene.tree.append_child(root, element);
    scene.tree.set_image(element, image, None);

    let frame = scene.render(renderer, 1.0);

    assert_eq!(frame.pixel(52, 34), [255, 0, 0, 255]);
}

#[test]
fn large_images_are_dropped_from_their_textures_at_the_byte_budget_and_uploaded_again() {
    let renderer = Renderer::with_options(&RendererOptions {
        image_byte_budget: FIELD_BYTES + FIELD_BYTES / 2,
        ..RendererOptions::default()
    })
    .expect("this Mac runs Metal 4");
    let first = decoded(&renderer, FIELD_PNG);
    let second = decoded(&renderer, FIELD_PNG);
    let mut scene = Scene::new(
        ".picture { position: absolute; left: 0; top: 0; width: 128px; height: 96px; }",
        Size {
            width: 200.0,
            height: 120.0,
        },
    );
    let element = scene.tree.create(Role::Image);
    scene.add_classes(element, &["picture"]);
    let root = scene.root();
    scene.tree.append_child(root, element);
    let mut target = OffscreenTarget::new(
        &renderer,
        Size {
            width: 200.0,
            height: 120.0,
        },
        1.0,
    );
    let mut show = |(image, size): (ImageId, Size)| {
        scene.tree.set_image(element, image, Some(size));
        scene.frame();
        target.render(scene.tree.display_list());
    };

    show(first);
    let after_first = renderer.image_statistics();
    show(second);
    let after_second = renderer.image_statistics();
    show(first);
    let after_third = renderer.image_statistics();

    assert_eq!((after_first.uploads, after_first.evictions), (1, 0));
    assert_eq!((after_second.uploads, after_second.evictions), (2, 1));
    assert_eq!((after_third.uploads, after_third.evictions), (3, 2));
    assert!(after_third.texture_bytes <= FIELD_BYTES + FIELD_BYTES / 2);
}

/// Fills a rectangle reaching far past the canvas.
struct SpillingPainter;

impl CanvasPainter for SpillingPainter {
    fn draw(&mut self, _canvas: CanvasId, encoder: &mut CanvasEncoder<'_>) {
        encoder.fill(
            Rect::new(-100.0, -100.0, 400.0, 400.0),
            Color {
                red: 0,
                green: 0,
                blue: 255,
                alpha: 1.0,
            },
        );
    }
}

#[test]
fn a_canvas_draw_stays_inside_the_content_box_of_its_element() {
    let renderer = renderer();
    let mut scene = Scene::new(
        ".drawing { position: absolute; left: 50px; top: 30px; width: 100px; height: 60px; }",
        Size {
            width: 200.0,
            height: 120.0,
        },
    );
    let element: ElementId = scene.tree.create(Role::Canvas);
    scene.add_classes(element, &["drawing"]);
    let root = scene.root();
    scene.tree.append_child(root, element);
    scene.tree.set_canvas(element, CanvasId(7));
    let mut target = OffscreenTarget::new(
        renderer,
        Size {
            width: 200.0,
            height: 120.0,
        },
        1.0,
    );
    target.set_canvas_painter(Box::new(SpillingPainter));
    scene.frame();

    let frame = target.render(scene.tree.display_list());

    assert_eq!(frame.pixel(100, 60), [0, 0, 255, 255]);
    assert_eq!(frame.pixel(51, 31), [0, 0, 255, 255]);
    assert_eq!(frame.pixel(49, 60), CLEAR);
    assert_eq!(frame.pixel(100, 91), CLEAR);
    assert_eq!(frame.pixel(10, 10), CLEAR);
}
