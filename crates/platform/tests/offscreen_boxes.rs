mod support;

use std::f32::consts::SQRT_2;
use std::sync::{LazyLock, mpsc};
use std::thread;
use std::time::Duration;

use morok_core::Size;
use morok_platform::{FrameImage, OffscreenTarget, Renderer};
use support::Scene;

const SURFACE: Size = Size {
    width: 200.0,
    height: 120.0,
};
const CLEAR: [u8; 4] = [0, 0, 0, 0];
const RED: [u8; 4] = [255, 0, 0, 255];
const GREEN: [u8; 4] = [0, 255, 0, 255];
const BLUE: [u8; 4] = [0, 0, 255, 255];
const BLACK: [u8; 4] = [0, 0, 0, 255];
const WHITE: [u8; 4] = [255, 255, 255, 255];

/// The largest difference of a channel that rounding to 8 bits and the arithmetic of the GPU
/// explain.
const CHANNEL_TOLERANCE: f32 = 2.0;

/// The box most scenes draw: 100 by 100 CSS pixels at (20, 10).
const PLACED: &str = "position: absolute; left: 20px; top: 10px; width: 100px; height: 100px;";

/// One renderer for every scene, as an application has one for every surface.
fn renderer() -> &'static Renderer {
    static RENDERER: LazyLock<Renderer> = LazyLock::new(|| Renderer::new().expect("this Mac runs Metal 4"));
    &RENDERER
}

/// Draws a scene whose root holds one element per class of `classes`, in that order.
fn render(rules: &str, classes: &[&str], scale: f32) -> FrameImage {
    let mut scene = Scene::new(rules, SURFACE);
    for class in classes {
        scene.add(scene.root(), &[class]);
    }
    scene.render(renderer(), scale)
}

fn assert_pixel_near(image: &FrameImage, x: usize, y: usize, expected: [f32; 4]) {
    let actual = image.pixel(x, y);
    let is_near = actual
        .iter()
        .zip(expected)
        .all(|(channel, expected)| (f32::from(*channel) - expected).abs() <= CHANNEL_TOLERANCE);
    assert!(
        is_near,
        "the pixel ({x}, {y}) is {actual:?}, expected about {expected:?}"
    );
}

/// The error function, by formula 7.1.26 of Abramowitz and Stegun.
fn error_function(value: f32) -> f32 {
    let magnitude = value.abs();
    let term = 1.0 / (1.0 + 0.327_591_1 * magnitude);
    let polynomial =
        ((((1.061_405_4 * term - 1.453_152_1) * term + 1.421_413_8) * term - 0.284_496_72) * term + 0.254_829_6) * term;
    (1.0 - polynomial * (-magnitude * magnitude).exp()).copysign(value)
}

/// How much of a Gaussian of the given deviation lies between two distances from its center.
fn gaussian_share(from: f32, to: f32, deviation: f32) -> f32 {
    let cumulative = |distance: f32| 0.5 + 0.5 * error_function(distance / (deviation * SQRT_2));
    cumulative(to) - cumulative(from)
}

/// The share of a pixel a disc covers, by the distance of the pixel center from the disc center.
fn disc_coverage(pixel_x: usize, pixel_y: usize, center: (f32, f32), radius: f32) -> f32 {
    let distance = (pixel_x as f32 + 0.5 - center.0).hypot(pixel_y as f32 + 0.5 - center.1);
    (0.5 - (distance - radius)).clamp(0.0, 1.0)
}

#[test]
fn a_background_fills_its_border_box_and_nothing_else() {
    let image = render(
        ".box { position: absolute; left: 20px; top: 10px; width: 100px; height: 50px; \
         background-color: var(--color-red); }",
        &["box"],
        1.0,
    );

    assert_eq!((image.width, image.height), (200, 120));
    assert_eq!(image.pixel(20, 10), RED);
    assert_eq!(image.pixel(119, 59), RED);
    assert_eq!(image.pixel(19, 10), CLEAR);
    assert_eq!(image.pixel(20, 9), CLEAR);
    assert_eq!(image.pixel(120, 59), CLEAR);
    assert_eq!(image.pixel(119, 60), CLEAR);
}

#[test]
fn an_empty_display_list_leaves_the_surface_transparent() {
    let image = render(".box { display: none; }", &[], 1.0);

    assert!(image.pixels.iter().all(|channel| *channel == 0));
}

#[test]
fn a_css_pixel_is_as_many_device_pixels_as_the_scale() {
    let image = render(
        ".box { position: absolute; left: 20px; top: 10px; width: 100px; height: 50px; \
         background-color: var(--color-red); }",
        &["box"],
        2.0,
    );

    assert_eq!((image.width, image.height), (400, 240));
    assert_eq!(image.pixel(40, 20), RED);
    assert_eq!(image.pixel(239, 119), RED);
    assert_eq!(image.pixel(39, 20), CLEAR);
    assert_eq!(image.pixel(240, 119), CLEAR);
}

#[test]
fn a_box_between_device_pixels_is_drawn_on_the_nearest_ones() {
    let rules = ".box { position: absolute; left: 20.4px; top: 10px; width: 99.4px; height: 50px; \
        background-color: var(--color-red); }";

    let coarse = render(rules, &["box"], 1.0);
    let fine = render(rules, &["box"], 2.0);

    assert_eq!([coarse.pixel(19, 30), coarse.pixel(20, 30)], [CLEAR, RED]);
    assert_eq!([coarse.pixel(119, 30), coarse.pixel(120, 30)], [RED, CLEAR]);
    assert_eq!([fine.pixel(40, 60), fine.pixel(41, 60)], [CLEAR, RED]);
    assert_eq!([fine.pixel(239, 60), fine.pixel(240, 60)], [RED, CLEAR]);
}

#[test]
fn a_translucent_color_is_blended_over_what_lies_under_it_as_srgb_values() {
    let image = render(
        ".paper { position: absolute; inset: 0px; background-color: var(--color-paper); }\n\
         .veil { position: absolute; left: 20px; top: 10px; width: 100px; height: 50px; \
         background-color: var(--color-veil); }",
        &["paper", "veil"],
        1.0,
    );

    assert_pixel_near(&image, 60, 30, [127.5, 127.5, 255.0, 255.0]);
    assert_eq!(image.pixel(150, 30), WHITE);
}

#[test]
fn a_translucent_color_on_a_transparent_surface_is_premultiplied() {
    let image = render(
        ".veil { position: absolute; left: 20px; top: 10px; width: 100px; height: 50px; \
         background-color: var(--color-veil); }",
        &["veil"],
        1.0,
    );

    assert_pixel_near(&image, 60, 30, [0.0, 0.0, 127.5, 127.5]);
}

#[test]
fn every_corner_is_cut_along_the_circle_of_its_own_radius() {
    let image = render(
        &format!(
            ".box {{ {PLACED} background-color: var(--color-red); \
             border-top-left-radius: var(--radius-small); border-top-right-radius: var(--radius-corner); \
             border-bottom-right-radius: var(--radius-large); }}"
        ),
        &["box"],
        1.0,
    );

    let corners = [
        ("top left", 10_usize, (20_usize, 10_usize), (30.0, 20.0)),
        ("top right", 20, (100, 10), (100.0, 30.0)),
        ("bottom right", 40, (80, 70), (80.0, 70.0)),
    ];
    for (corner, radius, (left, top), center) in corners {
        let mut partly_covered = 0;
        for y in top..top + radius {
            for x in left..left + radius {
                let coverage = disc_coverage(x, y, center, radius as f32);
                if coverage > 0.0 && coverage < 1.0 {
                    partly_covered += 1;
                }
                let expected = 255.0 * coverage;
                assert_pixel_near(&image, x, y, [expected, 0.0, 0.0, expected]);
            }
        }
        assert!(partly_covered > radius, "the {corner} corner has an anti-aliased arc");
    }
    assert_eq!(image.pixel(20, 109), RED, "the bottom left corner is square");
    assert_eq!(image.pixel(70, 60), RED);
}

#[test]
fn a_border_is_a_ring_of_its_widths_in_the_colors_of_its_sides() {
    let image = render(
        ".frame { position: absolute; left: 20px; top: 10px; width: 120px; height: 80px; \
         background-color: var(--color-paper); border-style: solid; border-width: 4px 8px 12px 16px; \
         border-color: var(--color-red) var(--color-green) var(--color-blue) var(--color-ink); }",
        &["frame"],
        1.0,
    );

    assert_eq!([image.pixel(80, 13), image.pixel(80, 14)], [RED, WHITE]);
    assert_eq!([image.pixel(131, 50), image.pixel(132, 50)], [WHITE, GREEN]);
    assert_eq!([image.pixel(80, 77), image.pixel(80, 78)], [WHITE, BLUE]);
    assert_eq!([image.pixel(35, 50), image.pixel(36, 50)], [BLACK, WHITE]);
    assert_eq!(image.pixel(140, 50), CLEAR);
}

#[test]
fn the_sides_of_a_border_meet_on_the_line_between_the_outer_and_the_inner_corner() {
    let image = render(
        ".frame { position: absolute; left: 20px; top: 10px; width: 120px; height: 80px; \
         border-style: solid; border-width: 4px 8px 12px 16px; \
         border-color: var(--color-red) var(--color-green) var(--color-blue) var(--color-ink); }",
        &["frame"],
        1.0,
    );

    assert_eq!(image.pixel(30, 11), RED, "above the line the top side owns the corner");
    assert_eq!(image.pixel(22, 13), BLACK, "below the line the left side owns it");
    assert_eq!(image.pixel(133, 11), RED);
    assert_eq!(image.pixel(138, 13), GREEN);
    assert_eq!(image.pixel(137, 80), GREEN);
    assert_eq!(image.pixel(133, 88), BLUE);
    assert_eq!(image.pixel(22, 80), BLACK);
    assert_eq!(image.pixel(34, 88), BLUE);
    let on_the_line = image.pixel(27, 11);
    assert!(
        on_the_line[0] > 0 && on_the_line[0] < 255 && on_the_line[3] == 255,
        "the line between two sides is anti-aliased, the pixel on it is {on_the_line:?}"
    );
}

#[test]
fn a_rounded_border_keeps_its_width_around_the_corner() {
    let image = render(
        &format!(".ring {{ {PLACED} border: 10px solid var(--color-red); border-radius: var(--radius-corner); }}"),
        &["ring"],
        1.0,
    );

    assert_eq!(image.pixel(22, 12), CLEAR, "outside the outer arc");
    assert_eq!(
        image.pixel(29, 19),
        RED,
        "between the arcs, 15 pixels from their center"
    );
    assert_eq!(image.pixel(36, 26), CLEAR, "inside the inner arc");
    for x in 20..40 {
        let on_the_diagonal = x - 10;
        let outer = disc_coverage(x, on_the_diagonal, (40.0, 30.0), 20.0);
        let inner = disc_coverage(x, on_the_diagonal, (40.0, 30.0), 10.0);
        let expected = 255.0 * outer.min(1.0 - inner);
        assert_pixel_near(&image, x, on_the_diagonal, [expected, 0.0, 0.0, expected]);
    }
}

#[test]
fn an_outline_is_a_ring_around_the_border_box_at_its_offset() {
    let image = render(
        ".box { position: absolute; left: 60px; top: 30px; width: 80px; height: 60px; \
         outline: 2px solid var(--color-red); outline-offset: 3px; }",
        &["box"],
        1.0,
    );

    assert_eq!(image.pixel(58, 60), CLEAR, "the gap of the offset");
    assert_eq!([image.pixel(55, 60), image.pixel(56, 60)], [RED, RED]);
    assert_eq!(image.pixel(54, 60), CLEAR);
    assert_eq!([image.pixel(100, 25), image.pixel(100, 26)], [RED, RED]);
    assert_eq!([image.pixel(100, 24), image.pixel(100, 27)], [CLEAR, CLEAR]);
    assert_eq!(image.pixel(100, 60), CLEAR, "an outline fills nothing");
}

#[test]
fn a_linear_gradient_runs_along_its_line_between_its_stops() {
    let image = render(
        &format!(".box {{ {PLACED} background-image: linear-gradient(90deg, var(--color-red), var(--color-blue)); }}"),
        &["box"],
        1.0,
    );

    for x in 20..120 {
        let position = (x as f32 - 20.0 + 0.5) / 100.0;
        assert_pixel_near(&image, x, 60, [255.0 * (1.0 - position), 0.0, 255.0 * position, 255.0]);
    }
    assert_eq!(
        image.pixel(60, 20),
        image.pixel(60, 100),
        "the color is the same across the line"
    );
}

#[test]
fn a_gradient_without_a_direction_runs_downwards() {
    let image = render(
        &format!(".box {{ {PLACED} background-image: linear-gradient(var(--color-red), var(--color-blue)); }}"),
        &["box"],
        1.0,
    );

    for y in 10..110 {
        let position = (y as f32 - 10.0 + 0.5) / 100.0;
        assert_pixel_near(&image, 60, y, [255.0 * (1.0 - position), 0.0, 255.0 * position, 255.0]);
    }
}

#[test]
fn stops_at_one_position_change_the_color_at_once() {
    let image = render(
        &format!(
            ".box {{ {PLACED} background-image: linear-gradient(90deg, var(--color-red) 25%, \
             var(--color-green) 25%, var(--color-green) 75%, var(--color-blue) 75%); }}"
        ),
        &["box"],
        1.0,
    );

    assert_eq!([image.pixel(20, 60), image.pixel(44, 60)], [RED, RED]);
    assert_eq!([image.pixel(45, 60), image.pixel(94, 60)], [GREEN, GREEN]);
    assert_eq!([image.pixel(95, 60), image.pixel(119, 60)], [BLUE, BLUE]);
}

#[test]
fn a_radial_gradient_runs_from_its_center_to_its_radius() {
    let image = render(
        &format!(
            ".box {{ {PLACED} background-image: radial-gradient(circle closest-side, var(--color-red), var(--color-blue)); }}"
        ),
        &["box"],
        1.0,
    );

    for x in 70..120 {
        let position = (x as f32 + 0.5 - 70.0).hypot(0.5) / 50.0;
        assert_pixel_near(&image, x, 60, [255.0 * (1.0 - position), 0.0, 255.0 * position, 255.0]);
    }
    assert_eq!(
        image.pixel(21, 11),
        BLUE,
        "past its radius the gradient keeps its last color"
    );
}

#[test]
fn a_gradient_tile_covers_the_padding_box_and_repeats_under_the_borders() {
    let image = render(
        ".box { position: absolute; left: 20px; top: 10px; width: 80px; height: 80px; \
         border: 10px solid transparent; \
         background-image: linear-gradient(90deg, var(--color-red), var(--color-blue)); }",
        &["box"],
        1.0,
    );

    let first_of_the_tile = [252.875, 0.0, 2.125, 255.0];
    let last_of_the_tile = [2.125, 0.0, 252.875, 255.0];
    assert_pixel_near(&image, 30, 60, first_of_the_tile);
    assert_pixel_near(&image, 89, 60, last_of_the_tile);
    assert_pixel_near(&image, 29, 60, last_of_the_tile);
    assert_pixel_near(&image, 90, 60, first_of_the_tile);
    assert_eq!(image.pixel(100, 60), CLEAR);
}

#[test]
fn an_outer_shadow_fades_as_the_gaussian_of_half_its_blur() {
    let image = render(
        ".card { position: absolute; left: 60px; top: 30px; width: 80px; height: 60px; \
         box-shadow: var(--shadow-soft); }",
        &["card"],
        1.0,
    );

    let deviation = 10.0;
    let rows = gaussian_share(-30.0, 30.0, deviation);
    let mut previous_alpha = u8::MAX;
    for x in 140..175 {
        let beyond_the_edge = x as f32 + 0.5 - 140.0;
        let expected = 255.0 * rows * gaussian_share(beyond_the_edge, beyond_the_edge + 80.0, deviation);
        assert_pixel_near(&image, x, 60, [0.0, 0.0, 0.0, expected]);
        let alpha = image.pixel(x, 60)[3];
        assert!(alpha <= previous_alpha, "the shadow never grows away from the box");
        previous_alpha = alpha;
    }
    assert!(
        image.pixel(140, 60)[3] > 115,
        "next to the box the shadow is at half its strength"
    );
    assert_eq!(
        image.pixel(172, 60),
        CLEAR,
        "three deviations away nothing is left of it"
    );
}

#[test]
fn a_shadow_fades_the_same_way_at_any_scale() {
    let scale = 2.0;
    let image = render(
        ".card { position: absolute; left: 60px; top: 30px; width: 80px; height: 60px; \
         box-shadow: var(--shadow-soft); }",
        &["card"],
        scale,
    );

    let deviation = 10.0;
    let rows = gaussian_share(-30.0, 30.0, deviation);
    for x in 280..340 {
        let beyond_the_edge = (x as f32 + 0.5) / scale - 140.0;
        let expected = 255.0 * rows * gaussian_share(beyond_the_edge, beyond_the_edge + 80.0, deviation);
        assert_pixel_near(&image, x, 120, [0.0, 0.0, 0.0, expected]);
    }
}

#[test]
fn the_box_hides_the_shadow_under_it() {
    let image = render(
        ".card { position: absolute; left: 60px; top: 30px; width: 80px; height: 60px; \
         box-shadow: var(--shadow-soft); }",
        &["card"],
        1.0,
    );

    assert_eq!(image.pixel(100, 60), CLEAR);
    assert_eq!(image.pixel(60, 30), CLEAR);
    assert_eq!(image.pixel(139, 89), CLEAR);
    assert!(image.pixel(59, 60)[3] > 115);
}

#[test]
fn a_shadow_moves_by_its_offset_and_grows_by_its_spread() {
    let image = render(
        ".card { position: absolute; left: 60px; top: 30px; width: 80px; height: 60px; \
         box-shadow: var(--shadow-hard); }",
        &["card"],
        1.0,
    );

    let half_black = [0.0, 0.0, 0.0, 127.5];
    assert_pixel_near(&image, 153, 99, half_black);
    assert_eq!([image.pixel(154, 99), image.pixel(153, 100)], [CLEAR, CLEAR]);
    assert_pixel_near(&image, 100, 95, half_black);
    assert_pixel_near(&image, 145, 32, half_black);
    assert_eq!(image.pixel(145, 31), CLEAR);
    assert_eq!(image.pixel(100, 60), CLEAR, "the box hides the shadow under it");
}

#[test]
fn a_shadow_follows_the_round_corners_of_its_box() {
    let image = render(
        ".card { position: absolute; left: 60px; top: 30px; width: 80px; height: 60px; \
         border-radius: var(--radius-corner); box-shadow: var(--shadow-hard); }",
        &["card"],
        1.0,
    );

    assert_eq!(image.pixel(152, 98), CLEAR, "the corner of the shadow is round");
    assert_pixel_near(&image, 145, 91, [0.0, 0.0, 0.0, 127.5]);
    for step in 0..24 {
        let (x, y) = (130 + step, 76 + step);
        let shadow = disc_coverage(x, y, (130.0, 76.0), 24.0);
        let hidden_by_the_box = disc_coverage(x, y, (120.0, 70.0), 20.0);
        let expected = 127.5 * shadow * (1.0 - hidden_by_the_box);
        assert_pixel_near(&image, x, y, [0.0, 0.0, 0.0, expected]);
    }
}

#[test]
fn an_inset_shadow_darkens_the_box_from_its_padding_edge_inwards() {
    let image = render(
        ".well { position: absolute; left: 60px; top: 30px; width: 80px; height: 60px; \
         background-color: var(--color-paper); box-shadow: var(--shadow-well); }",
        &["well"],
        1.0,
    );

    let deviation = 10.0;
    let rows = gaussian_share(-30.0, 30.0, deviation);
    for x in 60..100 {
        let inside_the_edge = x as f32 + 0.5 - 60.0;
        let light = rows * gaussian_share(-inside_the_edge, 80.0 - inside_the_edge, deviation);
        let expected = 255.0 * light;
        assert_pixel_near(&image, x, 60, [expected, expected, expected, 255.0]);
    }
    assert_eq!(image.pixel(59, 60), CLEAR, "an inset shadow stays inside the box");
    assert!(image.pixel(60, 60)[0] < 140, "at the edge half of the light is gone");
    assert!(image.pixel(99, 60)[0] > 245, "the middle of the box is lit");
}

#[test]
fn an_inset_shadow_with_a_spread_draws_a_rim_inside_the_border() {
    let image = render(
        ".well { position: absolute; left: 60px; top: 30px; width: 80px; height: 60px; \
         background-color: var(--color-paper); border: 2px solid var(--color-ink); box-shadow: var(--shadow-rim); }",
        &["well"],
        1.0,
    );

    assert_eq!([image.pixel(61, 60), image.pixel(62, 60)], [BLACK, RED]);
    assert_eq!([image.pixel(66, 60), image.pixel(67, 60)], [RED, WHITE]);
    assert_eq!(
        [image.pixel(100, 32), image.pixel(100, 36), image.pixel(100, 37)],
        [RED, RED, WHITE]
    );
}

#[test]
fn a_rectangular_clip_cuts_what_spills_out_of_the_padding_box() {
    let mut scene = Scene::new(
        ".frame { position: absolute; left: 20px; top: 10px; width: 100px; height: 60px; \
         border: 5px solid var(--color-ink); overflow: hidden; }\n\
         .spill { position: absolute; left: -30px; top: -30px; width: 300px; height: 300px; \
         background-color: var(--color-red); }",
        SURFACE,
    );
    let frame = scene.add(scene.root(), &["frame"]);
    scene.add(frame, &["spill"]);

    let image = scene.render(renderer(), 1.0);

    assert_eq!([image.pixel(24, 40), image.pixel(25, 40)], [BLACK, RED]);
    assert_eq!([image.pixel(114, 40), image.pixel(115, 40)], [RED, BLACK]);
    assert_eq!([image.pixel(60, 14), image.pixel(60, 15)], [BLACK, RED]);
    assert_eq!([image.pixel(60, 64), image.pixel(60, 65)], [RED, BLACK]);
    assert_eq!([image.pixel(60, 80), image.pixel(130, 40)], [CLEAR, CLEAR]);
}

#[test]
fn a_rounded_clip_cuts_along_the_padding_edge_of_its_corners() {
    let mut scene = Scene::new(
        ".frame { position: absolute; left: 20px; top: 10px; width: 100px; height: 100px; \
         border: 4px solid transparent; border-radius: var(--radius-corner); overflow: hidden; }\n\
         .spill { position: absolute; left: -30px; top: -30px; width: 300px; height: 300px; \
         background-color: var(--color-red); }",
        SURFACE,
    );
    let frame = scene.add(scene.root(), &["frame"]);
    scene.add(frame, &["spill"]);

    let image = scene.render(renderer(), 1.0);

    for y in 14..30 {
        for x in 24..40 {
            let expected = 255.0 * disc_coverage(x, y, (40.0, 30.0), 16.0);
            assert_pixel_near(&image, x, y, [expected, 0.0, 0.0, expected]);
        }
    }
    assert_eq!([image.pixel(23, 60), image.pixel(24, 60)], [CLEAR, RED]);
    assert_eq!(image.pixel(70, 60), RED);
}

#[test]
fn nested_rounded_clips_both_cut_what_they_enclose() {
    let mut scene = Scene::new(
        ".outer { position: absolute; left: 20px; top: 10px; width: 100px; height: 100px; \
         border-radius: var(--radius-large); overflow: hidden; }\n\
         .inner { position: absolute; left: 0px; top: 30px; width: 100px; height: 70px; \
         border-radius: var(--radius-corner); overflow: hidden; }\n\
         .spill { position: absolute; left: -50px; top: -50px; width: 300px; height: 300px; \
         background-color: var(--color-red); }",
        SURFACE,
    );
    let outer = scene.add(scene.root(), &["outer"]);
    let inner = scene.add(outer, &["inner"]);
    scene.add(inner, &["spill"]);

    let image = scene.render(renderer(), 1.0);

    for step in 0..20 {
        let (x, y) = (20 + step, 40 + step);
        let expected = 255.0 * disc_coverage(x, y, (40.0, 60.0), 20.0);
        assert_pixel_near(&image, x, y, [expected, 0.0, 0.0, expected]);
    }
    for step in 0..40 {
        let (x, y) = (80 + step, 70 + step);
        let expected = 255.0 * disc_coverage(x, y, (80.0, 70.0), 40.0);
        assert_pixel_near(&image, x, y, [expected, 0.0, 0.0, expected]);
    }
    assert_eq!(image.pixel(70, 39), CLEAR, "above the inner clip");
    assert_eq!(image.pixel(70, 70), RED);
}

#[test]
fn later_items_are_drawn_over_earlier_ones() {
    let image = render(
        ".under { position: absolute; left: 20px; top: 10px; width: 100px; height: 50px; \
         background-color: var(--color-red); }\n\
         .over { position: absolute; left: 60px; top: 30px; width: 100px; height: 50px; \
         background-color: var(--color-blue); }",
        &["under", "over"],
        1.0,
    );

    assert_eq!(image.pixel(40, 20), RED);
    assert_eq!(image.pixel(90, 45), BLUE);
    assert_eq!(image.pixel(140, 70), BLUE);
}

/// A line of text at (20, 10), and a box that either covers the text or stands beside it.
const TEXT_AND_BOX: &str = ".text { position: absolute; left: 20px; top: 10px; width: 160px; height: 40px; \
     color: var(--color-ink); font-family: var(--font-ui); font-size: var(--text-lg); line-height: var(--text-leading); }\
     .cover { position: absolute; left: 0; top: 0; width: 200px; height: 60px; background-color: var(--color-red); }\
     .beside { position: absolute; left: 20px; top: 70px; width: 160px; height: 40px; \
     background-color: var(--color-blue); }";

fn render_text_then(class: &str) -> FrameImage {
    let mut scene = Scene::new(TEXT_AND_BOX, SURFACE);
    let text = scene.add(scene.root(), &["text"]);
    scene.add_text(text, "MMMMMMMM");
    scene.add(scene.root(), &[class]);
    scene.render(renderer(), 1.0)
}

fn has_ink(image: &FrameImage, left: usize, top: usize, width: usize, height: usize) -> bool {
    (top..top + height).any(|y| (left..left + width).any(|x| image.pixel(x, y) != CLEAR && image.pixel(x, y) != RED))
}

#[test]
fn a_box_painted_after_a_text_covers_the_text() {
    let image = render_text_then("cover");

    assert_eq!(image.pixel(100, 30), RED);
    assert!(
        !has_ink(&image, 0, 0, 200, 60),
        "the text shows through the box over it"
    );
}

#[test]
fn a_box_painted_after_a_text_beside_it_leaves_the_text_as_it_is() {
    let alone = {
        let mut scene = Scene::new(TEXT_AND_BOX, SURFACE);
        let text = scene.add(scene.root(), &["text"]);
        scene.add_text(text, "MMMMMMMM");
        scene.render(renderer(), 1.0)
    };
    let with_box = render_text_then("beside");

    assert!(has_ink(&alone, 20, 10, 160, 40), "the text draws glyphs");
    for y in 0..60 {
        for x in 0..200 {
            assert_eq!(alone.pixel(x, y), with_box.pixel(x, y), "the pixel ({x}, {y}) changed");
        }
    }
    assert_eq!(with_box.pixel(100, 90), BLUE);
}

#[test]
fn a_frame_starts_from_a_cleared_surface() {
    let mut scene = Scene::new(
        ".box { position: absolute; left: 20px; top: 10px; width: 100px; height: 50px; \
         background-color: var(--color-red); }\n\
         .hidden { display: none; }",
        SURFACE,
    );
    let element = scene.add(scene.root(), &["box"]);
    let mut target = OffscreenTarget::new(renderer(), SURFACE, 1.0);
    scene.frame();
    let first = target.render(scene.tree.display_list());

    scene.add_classes(element, &["hidden"]);
    scene.frame();
    let second = target.render(scene.tree.display_list());

    assert_eq!(first.pixel(60, 30), RED);
    assert!(second.pixels.iter().all(|channel| *channel == 0));
}

#[test]
fn more_boxes_than_the_first_buffers_hold_are_all_drawn() {
    let mut scene = Scene::new(
        ".grid { display: grid; grid-template-columns: repeat(40, 5px); grid-auto-rows: 4px; }\n\
         .cell { background-color: var(--color-red); }\n\
         .last { background-color: var(--color-blue); }",
        SURFACE,
    );
    scene.add_classes(scene.root(), &["grid"]);
    for _ in 0..1199 {
        scene.add(scene.root(), &["cell"]);
    }
    scene.add(scene.root(), &["last"]);

    let image = scene.render(renderer(), 1.0);

    assert_eq!(image.pixel(0, 0), RED);
    assert_eq!(image.pixel(194, 119), RED);
    assert_eq!(image.pixel(195, 116), BLUE);
    assert_eq!(image.pixel(199, 119), BLUE);
}

#[test]
fn renderers_created_at_the_same_moment_all_come_up() {
    const RENDERERS: usize = 8;
    let (created, creations) = mpsc::channel();
    for _ in 0..RENDERERS {
        let created = created.clone();
        thread::spawn(move || created.send(Renderer::new().map(drop)));
    }

    for _ in 0..RENDERERS {
        let creation = creations
            .recv_timeout(Duration::from_secs(60))
            .expect("every renderer is created within a minute");
        assert_eq!(creation, Ok(()));
    }
}
