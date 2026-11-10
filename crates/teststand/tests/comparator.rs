use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use morok_core::{Rect, Size};
use morok_css::SourceFile;
use morok_platform::FrameImage;
use morok_teststand::{
    Case, CaseVerdict, ChromeBuild, CompiledSuite, ElementBox, Etalon, EtalonStore, Mismatch, Moment, MorokSide,
    Rendering, Scene, Stand, Suite, SuiteReport, Thresholds, check, compare,
};

const TESTSTAND: &str = env!("CARGO_BIN_EXE_morok-teststand");
const SCRATCH_ROOT: &str = env!("CARGO_TARGET_TMPDIR");

const CHANNELS: usize = 4;
const OPAQUE: u8 = 255;
const GRAY: [u8; 4] = [128, 128, 128, OPAQUE];
/// A gray whose CIEDE2000 difference from [`GRAY`] is 1.5: under the 2.0 a pixel may differ by.
const GRAY_NEARBY: [u8; 4] = [132, 132, 132, OPAQUE];
/// A gray whose CIEDE2000 difference from [`GRAY`] is 2.9: above the 2.0 a pixel may differ by.
const GRAY_APART: [u8; 4] = [136, 136, 136, OPAQUE];
const ACCENT: [u8; 4] = [20, 143, 120, OPAQUE];
const SIDE: usize = 100;

const CLEAN: u8 = 0;
const FAILURES: u8 = 1;
const COULD_NOT_RUN: Option<i32> = Some(2);

const PLATE: &str = ".plate { width: 80px; height: 40px; margin: var(--space-4); \
                     background-color: var(--color-accent); border-radius: var(--radius-md); }\n";
const TALLER_PLATE: &str = ".plate { width: 80px; height: 48px; margin: var(--space-4); \
                            background-color: var(--color-accent); border-radius: var(--radius-md); }\n";

fn thresholds() -> Thresholds {
    Thresholds::read(&Stand::of_workspace().thresholds_file).expect("the thresholds of the stand are read")
}

fn solid(width: usize, height: usize, color: [u8; 4]) -> FrameImage {
    FrameImage {
        width,
        height,
        pixels: color.repeat(width * height),
    }
}

/// The image with the pixels of the rectangle in the given color.
fn with_block(
    mut image: FrameImage,
    left: usize,
    top: usize,
    width: usize,
    height: usize,
    color: [u8; 4],
) -> FrameImage {
    for y in top..top + height {
        for x in left..left + width {
            let start = (y * image.width + x) * CHANNELS;
            image.pixels[start..start + CHANNELS].copy_from_slice(&color);
        }
    }
    image
}

/// A disc of the accent color on a transparent surface. The edge is smoothed by the share of
/// each pixel the disc covers, sampled on a grid of 8 by 8.
fn disc(side: usize, radius: f32) -> FrameImage {
    const SAMPLES: usize = 8;
    let center = side as f32 / 2.0;
    let mut pixels = Vec::with_capacity(side * side * CHANNELS);
    for y in 0..side {
        for x in 0..side {
            let covered = (0..SAMPLES * SAMPLES)
                .filter(|sample| {
                    let sample_x = x as f32 + ((sample % SAMPLES) as f32 + 0.5) / SAMPLES as f32;
                    let sample_y = y as f32 + ((sample / SAMPLES) as f32 + 0.5) / SAMPLES as f32;
                    (sample_x - center).hypot(sample_y - center) <= radius
                })
                .count();
            let coverage = covered as f32 / (SAMPLES * SAMPLES) as f32;
            pixels.extend(ACCENT.map(|channel| (f32::from(channel) * coverage).round() as u8));
        }
    }
    FrameImage {
        width: side,
        height: side,
        pixels,
    }
}

fn different_pixels(first: &FrameImage, second: &FrameImage) -> usize {
    first
        .pixels
        .chunks_exact(CHANNELS)
        .zip(second.pixels.chunks_exact(CHANNELS))
        .filter(|(first_pixel, second_pixel)| first_pixel != second_pixel)
        .count()
}

fn element_box(node: &str, x: f32, y: f32) -> ElementBox {
    ElementBox::new(node, Rect::new(x, y, 40.0, 20.0))
}

fn rendering(image: FrameImage, boxes: Vec<ElementBox>) -> Rendering {
    Rendering { image, boxes }
}

fn picture(image: FrameImage) -> Rendering {
    rendering(image, Vec::new())
}

fn boxes_alone(boxes: Vec<ElementBox>) -> Rendering {
    rendering(solid(SIDE, SIDE, GRAY), boxes)
}

fn plate(scene: &mut Scene) {
    scene.add(scene.root(), &["plate"]);
}

fn two_plates(scene: &mut Scene) {
    scene.add(scene.root(), &["plate"]);
    scene.add(scene.root(), &["plate"]);
}

/// A suite of two cases styled by the given stylesheet.
fn plates(stylesheet: &'static str) -> Suite {
    let viewport = Size {
        width: 120.0,
        height: 140.0,
    };
    Suite {
        name: "plates",
        component_stylesheets: Vec::new(),
        stylesheets: vec![SourceFile::new("plates.css", stylesheet)],
        cases: vec![
            Case {
                name: "one",
                viewport,
                build: plate,
                moment: Moment::AtRest,
            },
            Case {
                name: "two",
                viewport,
                build: two_plates,
                moment: Moment::AtRest,
            },
        ],
    }
}

/// A stand of its own for one test: the thresholds and the Chrome build of the workspace,
/// and scratch directories for etalons and diff images.
fn scratch_stand(test_name: &str) -> Stand {
    let scratch = Path::new(SCRATCH_ROOT).join("comparator").join(test_name);
    if scratch.exists() {
        fs::remove_dir_all(&scratch).expect("the scratch of the previous run is removed");
    }
    Stand {
        etalons_directory: scratch.join("etalons"),
        diffs_directory: scratch.join("diffs"),
        ..Stand::of_workspace()
    }
}

/// Stores, as the etalon of every case, what the kit itself draws of it. A test needs no
/// Chrome this way, and the kit matches the stand until the test changes one of the two.
fn store_what_the_kit_draws(suite: &Suite, stand: &Stand) {
    let chrome_build = ChromeBuild::read(&stand.chrome_build_file).expect("the Chrome build is read");
    let compiled = CompiledSuite::compile(suite).expect("the suite compiles");
    let store = EtalonStore::new(&stand.etalons_directory);
    let mut kit = MorokSide::new().expect("the kit has a renderer");
    for case in &suite.cases {
        let etalon = Etalon {
            manifest: compiled.manifest(case, &chrome_build),
            rendering: kit.render_case(&compiled, case),
        };
        store
            .write(suite.name, case.name, &etalon)
            .expect("the etalon is written");
    }
}

/// Changes the stored etalon of the case.
fn change_etalon(suite: &Suite, case_name: &str, stand: &Stand, change: impl FnOnce(&mut Etalon)) {
    let store = EtalonStore::new(&stand.etalons_directory);
    let mut etalon = store
        .read(suite.name, case_name)
        .expect("the etalon is read")
        .expect("the case has an etalon");
    change(&mut etalon);
    store
        .write(suite.name, case_name, &etalon)
        .expect("the etalon is written");
}

fn checked(suite: &Suite, stand: &Stand) -> SuiteReport {
    check(suite, stand).expect("the stand runs the check")
}

fn diff_image(stand: &Stand, name: &str) -> PathBuf {
    stand.diffs_directory.join("plates").join(name)
}

#[test]
fn a_box_more_than_half_a_pixel_away_fails_and_names_its_node() {
    let etalon = boxes_alone(vec![
        element_box("#0 root", 0.0, 0.0),
        element_box("#1 .badge", 10.0, 10.0),
    ]);
    let shifted = boxes_alone(vec![
        element_box("#0 root", 0.0, 0.0),
        element_box("#1 .badge", 11.0, 10.0),
    ]);

    let comparison = compare(&etalon, &shifted, &thresholds());

    assert_eq!(
        comparison.mismatches,
        [Mismatch::ElementBox {
            etalon: element_box("#1 .badge", 10.0, 10.0),
            candidate: element_box("#1 .badge", 11.0, 10.0),
        }]
    );
    assert!(comparison.mismatches[0].to_string().contains("`#1 .badge`"));
}

#[test]
fn a_box_half_a_pixel_away_is_within_the_tolerance() {
    let etalon = boxes_alone(vec![element_box("#0 root", 10.0, 10.0)]);
    let nudged = boxes_alone(vec![element_box("#0 root", 10.5, 9.5)]);

    assert!(compare(&etalon, &nudged, &thresholds()).matches());
}

#[test]
fn an_etalon_with_one_more_box_than_the_tree_fails() {
    let etalon = boxes_alone(vec![
        element_box("#0 root", 0.0, 0.0),
        element_box("#1 .badge", 10.0, 10.0),
        element_box("#2 .badge", 60.0, 10.0),
    ]);
    let tree = boxes_alone(vec![
        element_box("#0 root", 0.0, 0.0),
        element_box("#1 .badge", 10.0, 10.0),
    ]);

    assert_eq!(
        compare(&etalon, &tree, &thresholds()).mismatches,
        [Mismatch::ElementCount {
            etalon: 3,
            candidate: 2
        }]
    );
    assert_eq!(
        compare(&tree, &etalon, &thresholds()).mismatches,
        [Mismatch::ElementCount {
            etalon: 2,
            candidate: 3
        }]
    );
}

/// Two renderers smooth the edge of the same disc differently. Counted, the pixels of the edge
/// would fail the case; told as anti-aliasing, they leave it matching.
#[test]
fn pixels_that_differ_only_along_a_rounded_edge_do_not_count() {
    let etalon = disc(128, 40.0);
    let smoothed_differently = disc(128, 40.4);
    let thresholds = thresholds();
    let along_the_edge = different_pixels(&etalon, &smoothed_differently);
    assert!(
        along_the_edge as f32 > thresholds.mismatch_fraction * (128 * 128) as f32,
        "{along_the_edge} pixels differ"
    );

    let comparison = compare(&picture(etalon), &picture(smoothed_differently), &thresholds);

    assert_eq!(comparison.mismatches, []);
    assert_eq!(comparison.mismatching_pixels(), 0);
}

#[test]
fn a_pixel_mismatches_when_it_differs_by_more_than_two() {
    let etalon = picture(solid(SIDE, SIDE, GRAY));
    let thresholds = thresholds();
    let block = |color| picture(with_block(solid(SIDE, SIDE, GRAY), 40, 40, 5, 5, color));

    let apart = compare(&etalon, &block(GRAY_APART), &thresholds);
    let nearby = compare(&etalon, &block(GRAY_NEARBY), &thresholds);

    assert_eq!(apart.mismatching_pixels(), 25);
    assert_eq!(nearby.mismatching_pixels(), 0);
    assert!(
        apart.matches() && nearby.matches(),
        "a quarter of a percent may mismatch"
    );
}

#[test]
fn the_difference_of_a_pixel_is_measured_by_ciede2000() {
    let etalon = picture(solid(SIDE, SIDE, GRAY));

    let apart = compare(&etalon, &picture(solid(SIDE, SIDE, GRAY_APART)), &thresholds());
    let nearby = compare(&etalon, &picture(solid(SIDE, SIDE, GRAY_NEARBY)), &thresholds());

    assert!(
        (apart.mean_difference() - 2.947).abs() < 0.001,
        "{}",
        apart.mean_difference()
    );
    assert!(
        (nearby.mean_difference() - 1.495).abs() < 0.001,
        "{}",
        nearby.mean_difference()
    );
}

#[test]
fn a_translucent_pixel_differs_as_much_as_it_does_on_the_backdrop_that_shows_it() {
    let faint_shadow = [0, 0, 0, 20];
    let stronger_shadow = [0, 0, 0, 40];
    let etalon = picture(solid(SIDE, SIDE, faint_shadow));

    let comparison = compare(&etalon, &picture(solid(SIDE, SIDE, stronger_shadow)), &thresholds());

    assert_eq!(comparison.mismatching_pixels(), SIDE * SIDE);
}

#[test]
fn a_case_fails_when_more_than_half_a_percent_of_it_mismatches() {
    let etalon = picture(solid(SIDE, SIDE, GRAY));
    let one_percent_apart = picture(with_block(solid(SIDE, SIDE, GRAY), 30, 30, 10, 10, GRAY_APART));
    let half_a_percent_apart = picture(with_block(solid(SIDE, SIDE, GRAY), 30, 30, 10, 5, GRAY_APART));

    assert_eq!(
        compare(&etalon, &one_percent_apart, &thresholds()).mismatches,
        [Mismatch::MismatchingPixels {
            pixels: 100,
            area: 10_000
        }]
    );
    assert!(compare(&etalon, &half_a_percent_apart, &thresholds()).matches());
}

/// No pixel differs by more than a pixel may, and yet the whole case is off.
#[test]
fn a_case_fails_when_its_mean_difference_is_above_one() {
    let etalon = picture(solid(SIDE, SIDE, GRAY));
    let nearby_all_over = picture(solid(SIDE, SIDE, GRAY_NEARBY));

    let comparison = compare(&etalon, &nearby_all_over, &thresholds());

    assert_eq!(comparison.mismatching_pixels(), 0);
    assert!(
        matches!(comparison.mismatches[..], [Mismatch::MeanDifference { mean }] if (mean - 1.495).abs() < 0.001),
        "{:?}",
        comparison.mismatches
    );
}

#[test]
fn an_etalon_one_pixel_wider_than_the_rendering_fails() {
    let etalon = picture(solid(SIDE + 1, SIDE, GRAY));
    let narrower = picture(solid(SIDE, SIDE, GRAY));

    let comparison = compare(&etalon, &narrower, &thresholds());

    assert!(
        comparison.mismatches.contains(&Mismatch::ImageSize {
            etalon: (SIDE + 1, SIDE),
            candidate: (SIDE, SIDE)
        }),
        "{:?}",
        comparison.mismatches
    );
}

#[test]
fn an_etalon_comes_back_from_the_store_as_it_was_written() {
    let suite = plates(PLATE);
    let stand = scratch_stand("store");
    store_what_the_kit_draws(&suite, &stand);
    let chrome_build = ChromeBuild::read(&stand.chrome_build_file).expect("the Chrome build is read");
    let compiled = CompiledSuite::compile(&suite).expect("the suite compiles");
    let case = &suite.cases[1];

    let stored = EtalonStore::new(&stand.etalons_directory)
        .read(suite.name, case.name)
        .expect("the etalon is read");

    let drawn_again = Etalon {
        manifest: compiled.manifest(case, &chrome_build),
        rendering: MorokSide::new()
            .expect("the kit has a renderer")
            .render_case(&compiled, case),
    };
    assert_eq!(stored, Some(drawn_again));
}

#[test]
fn a_run_without_a_failing_case_exits_with_zero_and_one_with_a_failing_case_does_not() {
    let suite = plates(PLATE);
    let stand = scratch_stand("exit-codes");
    store_what_the_kit_draws(&suite, &stand);

    let clean = checked(&suite, &stand);

    assert_eq!(
        clean.cases,
        [
            (String::from("one"), CaseVerdict::Matches),
            (String::from("two"), CaseVerdict::Matches)
        ]
    );
    assert_eq!(clean.exit_code(), CLEAN);

    change_etalon(&suite, "two", &stand, |etalon| {
        etalon.rendering.light.boxes[1].x += 1.0;
    });
    let failing = checked(&suite, &stand);

    assert_eq!(failing.verdict("one"), Some(&CaseVerdict::Matches));
    assert!(failing.verdict("two").is_some_and(CaseVerdict::is_failure));
    assert_eq!(failing.exit_code(), FAILURES);
}

#[test]
fn a_case_that_matches_in_light_and_differs_in_dark_fails_and_leaves_a_diff_image() {
    let suite = plates(PLATE);
    let stand = scratch_stand("both-schemes");
    store_what_the_kit_draws(&suite, &stand);
    change_etalon(&suite, "one", &stand, |etalon| {
        let dark = &mut etalon.rendering.dark.image;
        *dark = with_block(dark.clone(), 60, 60, 40, 40, GRAY);
    });

    let report = checked(&suite, &stand);

    let Some(CaseVerdict::Differs { light, dark }) = report.verdict("one") else {
        panic!("the case differs: {report}");
    };
    assert_eq!(light, &[]);
    assert!(
        matches!(dark[..], [Mismatch::MismatchingPixels { pixels: 1600, .. }, ..]),
        "{dark:?}"
    );
    assert!(diff_image(&stand, "one-dark.png").is_file());
    assert!(!diff_image(&stand, "one-light.png").exists());
    assert!(!diff_image(&stand, "two-dark.png").exists());
}

/// The etalons were rendered from a stylesheet the suite no longer has. The kit draws the
/// plate of the new stylesheet, which no longer looks like the etalon, but the case is not
/// compared at all: it is stale.
#[test]
fn a_case_whose_stylesheet_changed_is_stale_before_it_is_compared() {
    let stand = scratch_stand("stale");
    store_what_the_kit_draws(&plates(PLATE), &stand);

    let report = checked(&plates(TALLER_PLATE), &stand);

    let stale = CaseVerdict::Stale {
        changed: vec!["stylesheet"],
    };
    assert_eq!(report.verdict("one"), Some(&stale));
    assert_eq!(report.verdict("two"), Some(&stale));
    assert_eq!(report.exit_code(), FAILURES);
    assert!(!diff_image(&stand, "one-light.png").exists());
    assert!(!diff_image(&stand, "one-dark.png").exists());
}

#[test]
fn a_case_chrome_has_not_rendered_fails() {
    let suite = plates(PLATE);
    let stand = scratch_stand("missing");

    let report = checked(&suite, &stand);

    assert_eq!(report.verdict("one"), Some(&CaseVerdict::MissingEtalon));
    assert_eq!(report.exit_code(), FAILURES);
}

#[test]
fn a_command_the_stand_does_not_have_exits_with_two() {
    let exit_code = |arguments: &[&str]| {
        Command::new(TESTSTAND)
            .args(arguments)
            .output()
            .expect("the stand runs")
            .status
            .code()
    };

    assert_eq!(exit_code(&[]), COULD_NOT_RUN);
    assert_eq!(exit_code(&["check"]), COULD_NOT_RUN);
    assert_eq!(exit_code(&["check", "no-such-suite"]), COULD_NOT_RUN);
    assert_eq!(exit_code(&["etalons", "paint", "primitives"]), COULD_NOT_RUN);
}
