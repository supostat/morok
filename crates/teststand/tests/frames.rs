use std::fs;
use std::io::Cursor;
use std::path::{Path, PathBuf};
use std::process::Command;

use morok_platform::FrameImage;
use morok_style::ColorScheme;
use morok_teststand::{
    Case, CompiledSuite, DEVICE_SCALE, MorokSide, Rendering, Stand, Suite, Thresholds, compare, suites,
};

const TESTSTAND: &str = env!("CARGO_BIN_EXE_morok-teststand");
const SCRATCH_ROOT: &str = env!("CARGO_TARGET_TMPDIR");
const LIST_FILE: &str = "frames.toml";

const WRITTEN: Option<i32> = Some(0);
const COULD_NOT_RUN: Option<i32> = Some(2);

const SUITE: &str = "button";
const CASE: &str = "primary-states";
const OTHER_CASE: &str = "default-states";
const LIGHT: &str = "light";
const DARK: &str = "dark";

const CHANNELS: usize = 4;
const ALPHA: usize = 3;
const OPAQUE: u32 = 255;

/// What the command left behind: how it exited, what it printed and what it complained of.
struct FramesRun {
    exit_code: Option<i32>,
    printed: String,
    complaint: String,
}

/// The directory of one test, empty when the test starts.
fn scratch(test_name: &str) -> PathBuf {
    let directory = Path::new(SCRATCH_ROOT).join("frames").join(test_name);
    if directory.exists() {
        fs::remove_dir_all(&directory).expect("the scratch of the previous run is removed");
    }
    fs::create_dir_all(&directory).expect("the scratch of the test is created");
    directory
}

/// One frame of a list, as the list writes it.
fn entry(suite: &str, case: &str, color_scheme: &str, file: &str) -> String {
    format!("[[frame]]\nsuite = \"{suite}\"\ncase = \"{case}\"\ncolor_scheme = \"{color_scheme}\"\nfile = \"{file}\"\n")
}

fn stand_run(arguments: &[&str]) -> FramesRun {
    let output = Command::new(TESTSTAND)
        .args(arguments)
        .output()
        .expect("the stand runs");
    FramesRun {
        exit_code: output.status.code(),
        printed: String::from_utf8(output.stdout).expect("the stand prints text"),
        complaint: String::from_utf8(output.stderr).expect("the stand complains in text"),
    }
}

/// Writes a list of the text into the directory and runs the command on it.
fn frames_of_list(directory: &Path, list: &str) -> FramesRun {
    let list_file = directory.join(LIST_FILE);
    fs::write(&list_file, list).expect("the list is written");
    stand_run(&["frames", list_file.to_str().expect("the scratch has a name in text")])
}

fn suite() -> Suite {
    suites::named(SUITE).expect("the stand has the suite of the buttons")
}

fn case_of(suite: &Suite) -> Case {
    suite
        .cases
        .iter()
        .find(|case| case.name == CASE)
        .copied()
        .expect("the suite of the buttons has the case of the primary button")
}

/// The frame a PNG of eight-bit RGBA holds, its colors multiplied by alpha as a frame holds
/// them.
fn frame_in(path: &Path) -> FrameImage {
    let png = fs::read(path).expect("the frame is a file");
    let mut reader = png::Decoder::new(Cursor::new(png))
        .read_info()
        .expect("the frame is a PNG");
    let mut pixels = vec![0; reader.output_buffer_size().expect("the frame fits in memory")];
    let frame = reader.next_frame(&mut pixels).expect("the PNG holds a frame");
    assert_eq!(
        (frame.color_type, frame.bit_depth),
        (png::ColorType::Rgba, png::BitDepth::Eight)
    );
    pixels.truncate(frame.buffer_size());
    for pixel in pixels.chunks_exact_mut(CHANNELS) {
        let alpha = u32::from(pixel[ALPHA]);
        for channel in &mut pixel[..ALPHA] {
            *channel = ((u32::from(*channel) * alpha + OPAQUE / 2) / OPAQUE) as u8;
        }
    }
    FrameImage {
        width: frame.width as usize,
        height: frame.height as usize,
        pixels,
    }
}

#[test]
fn a_frame_is_written_at_its_file_under_the_directory_of_the_list() {
    let directory = scratch("deep-file");

    let run = frames_of_list(&directory, &entry(SUITE, CASE, DARK, "deep/er/primary.png"));

    assert_eq!(run.exit_code, WRITTEN, "{}", run.complaint);
    let viewport = case_of(&suite()).viewport;
    let frame = frame_in(&directory.join("deep/er/primary.png"));
    assert_eq!(
        (frame.width, frame.height),
        (
            (viewport.width * DEVICE_SCALE) as usize,
            (viewport.height * DEVICE_SCALE) as usize
        )
    );
}

#[test]
fn a_frame_is_what_the_kit_draws_of_the_case_under_the_scheme_its_entry_names() {
    let directory = scratch("schemes");
    let list = [
        entry(SUITE, CASE, LIGHT, "light.png"),
        entry(SUITE, CASE, DARK, "dark.png"),
    ]
    .concat();

    let run = frames_of_list(&directory, &list);

    assert_eq!(run.exit_code, WRITTEN, "{}", run.complaint);
    assert_ne!(
        fs::read(directory.join("light.png")).expect("the light frame is written"),
        fs::read(directory.join("dark.png")).expect("the dark frame is written")
    );
    let suite = suite();
    let compiled = CompiledSuite::compile(&suite).expect("the suite compiles");
    let drawn = MorokSide::new()
        .expect("the kit has a renderer")
        .render_case(&compiled, &case_of(&suite));
    let thresholds =
        Thresholds::read(&Stand::of_workspace().thresholds_file).expect("the thresholds of the stand are read");
    for (color_scheme, file) in [(ColorScheme::Light, "light.png"), (ColorScheme::Dark, "dark.png")] {
        let expected = drawn.under(color_scheme);
        let written = Rendering {
            image: frame_in(&directory.join(file)),
            boxes: expected.boxes.clone(),
        };
        let comparison = compare(expected, &written, &thresholds);
        assert!(comparison.matches(), "{file}: {:?}", comparison.mismatches);
    }
}

#[test]
fn an_entry_the_stand_cannot_resolve_is_refused_before_anything_is_drawn() {
    for (test_name, unresolved, unknown) in [
        (
            "unknown-suite",
            entry("no-such-suite", CASE, DARK, "second.png"),
            "no-such-suite",
        ),
        (
            "unknown-case",
            entry(SUITE, "no-such-case", DARK, "second.png"),
            "no-such-case",
        ),
        ("unknown-scheme", entry(SUITE, CASE, "sepia", "second.png"), "sepia"),
    ] {
        let directory = scratch(test_name);
        let list = [entry(SUITE, CASE, DARK, "first.png"), unresolved].concat();

        let run = frames_of_list(&directory, &list);

        assert_eq!(run.exit_code, COULD_NOT_RUN, "{test_name}");
        assert!(run.complaint.contains(unknown), "{test_name}: {}", run.complaint);
        assert!(!directory.join("first.png").exists(), "{test_name}");
    }
}

#[test]
fn a_file_outside_the_directory_of_the_list_is_refused() {
    let scratch = scratch("outside");
    let directory = scratch.join("lists");
    fs::create_dir_all(&directory).expect("the directory of the list is created");
    let absolute = scratch.join("absolute.png");

    for file in [
        "../climbed.png",
        absolute.to_str().expect("the scratch has a name in text"),
    ] {
        let run = frames_of_list(&directory, &entry(SUITE, CASE, DARK, file));

        assert_eq!(run.exit_code, COULD_NOT_RUN, "{file}");
        assert!(run.complaint.contains(file), "{file}: {}", run.complaint);
    }
    assert!(!scratch.join("climbed.png").exists());
    assert!(!absolute.exists());
}

#[test]
fn a_list_that_cannot_be_read_as_one_is_refused_by_its_name() {
    let directory = scratch("unreadable");
    let misspelt = entry(SUITE, CASE, DARK, "frame.png").replace("color_scheme", "colour_scheme");

    let run = frames_of_list(&directory, &misspelt);

    assert_eq!(run.exit_code, COULD_NOT_RUN);
    assert!(run.complaint.contains(LIST_FILE), "{}", run.complaint);
    assert!(!directory.join("frame.png").exists());

    let absent = directory.join("absent.toml");
    let run = stand_run(&["frames", absent.to_str().expect("the scratch has a name in text")]);

    assert_eq!(run.exit_code, COULD_NOT_RUN);
    assert!(run.complaint.contains("absent.toml"), "{}", run.complaint);
}

#[test]
fn the_command_prints_how_many_frames_it_wrote_and_a_wrong_command_line_prints_the_usage() {
    let directory = scratch("count");
    let list = [
        entry(SUITE, CASE, DARK, "primary.png"),
        entry(SUITE, OTHER_CASE, DARK, "default.png"),
    ]
    .concat();

    let run = frames_of_list(&directory, &list);

    assert_eq!(run.exit_code, WRITTEN, "{}", run.complaint);
    assert_eq!(run.printed.trim_end(), "2 frames written");

    let run = stand_run(&["frames"]);

    assert_eq!(run.exit_code, COULD_NOT_RUN);
    assert!(run.complaint.contains("usage:"), "{}", run.complaint);
}
