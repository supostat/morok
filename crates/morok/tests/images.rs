mod support;

use std::borrow::Cow;
use std::process::ExitCode;
use std::time::{Duration, Instant};

use morok::components::Image;
use morok::core::accesskit::{NodeId, Role as AccessibleRole};
use morok::core::{ElementId, ImageId, ImageStatus, Rect, Size};
use morok::reactive::Signal;
use morok::style::{Declaration, LengthPercentage, LengthPercentageAuto, Pixels};
use morok::{Application, FrameImage, Styles, Window, WindowOptions};
use support::Expectations;

const RED_PNG: &[u8] = include_bytes!("images/red.png");
const BLUE_JPEG: &[u8] = include_bytes!("images/blue.jpg");
const UNREADABLE: &[u8] = b"these bytes are no picture";
const RED_SIZE: Size = Size {
    width: 40.0,
    height: 30.0,
};
const BLUE_SIZE: Size = Size {
    width: 30.0,
    height: 40.0,
};
const RED: [u8; 3] = [210, 60, 50];
const BLUE: [u8; 3] = [40, 90, 200];
/// How far a channel of a pixel may stray from the color of the picture: the JPEG is lossy.
const COLOR_TOLERANCE: u8 = 12;
/// The box the style of the test gives the unreadable picture.
const BROKEN_BOX: Size = Size {
    width: 50.0,
    height: 20.0,
};
const CONTENT_SIZE: Size = Size {
    width: 300.0,
    height: 200.0,
};
/// Long enough for a window to come on screen and for its first frames to be drawn.
const SETTLING: Duration = Duration::from_millis(1500);
/// How long the decoder has at most for the pictures of the test.
const DECODING: Duration = Duration::from_secs(5);

/// What the root component of the window built.
#[derive(Clone, Copy, Debug)]
struct Built {
    red: ElementId,
    /// The image whose signal is written the blue picture once the window is on screen.
    second: ElementId,
    broken: ElementId,
}

struct Stage {
    application: Application,
    window: Window,
    expectations: Expectations,
    built: Built,
    red: ImageId,
    unreadable: ImageId,
    second_picture: Signal<ImageId>,
}

impl Stage {
    /// Adds the red PNG and the unreadable bytes, and opens a window on three images: the red
    /// picture, a second image that shows the red picture until the test writes it another,
    /// and the unreadable picture in a box its inline style gives it.
    fn open() -> Self {
        let application = Application::new(Styles::of_kit()).expect("this Mac runs Metal 4");
        let red = application.add_image(RED_PNG.to_vec());
        let unreadable = application.add_image(UNREADABLE.to_vec());
        let second_picture = application.update(|runtime| runtime.signal(red));
        let options = WindowOptions {
            title: String::from("morok: images"),
            content_size: CONTENT_SIZE,
            blur_behind: false,
        };
        let mut built = None;
        let window = application.open_window(&options, |mount| {
            let red = Image::new(red, "Red block").mount(mount);
            let second = Image::new(second_picture, "Second block").mount(mount);
            let broken = Image::new(unreadable, "Broken picture").mount(mount);
            mount
                .build(broken)
                .inline(Declaration::Width(pixels(BROKEN_BOX.width)))
                .inline(Declaration::Height(pixels(BROKEN_BOX.height)));
            built = Some(Built { red, second, broken });
            mount.element().child(red).child(second).child(broken).id()
        });
        Self {
            application,
            window,
            expectations: Expectations::default(),
            built: built.expect("the window mounted its root component"),
            red,
            unreadable,
            second_picture,
        }
    }

    /// Runs the event loop until the decoder is done with the picture or the time for it is up.
    fn settled_status(&self, image: ImageId) -> ImageStatus {
        let started = Instant::now();
        loop {
            let status = self.application.image_status(image);
            if status != ImageStatus::Decoding || started.elapsed() > DECODING {
                return status;
            }
            self.application.run_for(Duration::from_millis(20));
        }
    }

    fn border_box(&self, element: ElementId) -> Rect {
        self.window.with_tree(|tree| tree.border_box(element))
    }

    /// The color of the pixel at the middle of the box of the element in the frame the window
    /// drew last.
    fn color_in_the_middle_of(&self, frame: &FrameImage, element: ElementId) -> [u8; 3] {
        let border_box = self.border_box(element);
        let scale = frame.width as f32 / CONTENT_SIZE.width;
        let x = ((border_box.x + border_box.width / 2.0) * scale) as usize;
        let y = ((border_box.y + border_box.height / 2.0) * scale) as usize;
        let [red, green, blue, _] = frame.pixel(x, y);
        [red, green, blue]
    }

    fn the_application_reads_the_status_of_every_picture(&mut self) {
        let statuses = [self.settled_status(self.red), self.settled_status(self.unreadable)];
        self.expectations.expect(
            statuses == [ImageStatus::Decoded(RED_SIZE), ImageStatus::Unreadable],
            "the PNG decodes at its size and the bytes that are no picture are unreadable",
            &format!("{statuses:?}"),
        );
    }

    fn a_decoded_picture_shows_at_its_natural_size(&mut self) {
        self.application.run_for(SETTLING);
        let statistics = self.window.frame_statistics();
        self.expectations.expect(
            statistics.presented_frames >= 1,
            "the window presents a first frame; is the display on?",
            &format!("{statistics:?}"),
        );
        let red_box = self.border_box(self.built.red);
        self.expectations.expect(
            red_box.size() == RED_SIZE,
            "the image takes the natural size of its picture",
            &format!("{red_box:?}"),
        );
        let shown = self.color_in_the_middle_of(&self.window.frame_image(), self.built.red);
        self.expectations.expect(
            is_close_to(shown, RED),
            "the frame of the window holds the pixels of the picture",
            &format!("{shown:?}"),
        );
    }

    fn an_unreadable_picture_leaves_its_box_and_its_name(&mut self) {
        let broken_box = self.border_box(self.built.broken);
        self.expectations.expect(
            broken_box.size() == BROKEN_BOX,
            "an image of an unreadable picture keeps the box its style gives it",
            &format!("{broken_box:?}"),
        );
        let broken = self.built.broken;
        let node = self.window.with_tree(|tree| {
            tree.accessibility_tree()
                .nodes
                .into_iter()
                .find(|(id, _)| *id == NodeId::from(broken))
                .map(|(_, node)| (node.role(), node.label().map(str::to_owned)))
        });
        self.expectations.expect(
            node == Some((AccessibleRole::Image, Some(String::from("Broken picture")))),
            "an assistive technology reads the image of an unreadable picture by its name",
            &format!("{node:?}"),
        );
    }

    fn a_picture_added_while_the_window_shows_takes_its_size_once_decoded(&mut self) {
        let blue = self.application.add_image(BLUE_JPEG.to_vec());
        self.application
            .update(|runtime| runtime.set(self.second_picture, blue));
        let status = self.settled_status(blue);
        self.application.run_for(SETTLING);
        let second_box = self.border_box(self.built.second);
        self.expectations.expect(
            status == ImageStatus::Decoded(BLUE_SIZE) && second_box.size() == BLUE_SIZE,
            "the JPEG the image is written while the window shows decodes and the image takes its size",
            &format!("{status:?}, {second_box:?}"),
        );
        let shown = self.color_in_the_middle_of(&self.window.frame_image(), self.built.second);
        self.expectations.expect(
            is_close_to(shown, BLUE),
            "the frame of the window holds the pixels of the JPEG",
            &format!("{shown:?}"),
        );
    }
}

fn pixels(length: f32) -> LengthPercentageAuto {
    LengthPercentageAuto::LengthPercentage(LengthPercentage {
        pixels: Pixels(length),
        fraction: None,
        tokens: Cow::Borrowed(&[]),
    })
}

fn is_close_to(shown: [u8; 3], expected: [u8; 3]) -> bool {
    shown
        .iter()
        .zip(expected)
        .all(|(shown, expected)| shown.abs_diff(expected) <= COLOR_TOLERANCE)
}

fn main() -> ExitCode {
    let mut stage = Stage::open();

    stage.the_application_reads_the_status_of_every_picture();
    stage.a_decoded_picture_shows_at_its_natural_size();
    stage.an_unreadable_picture_leaves_its_box_and_its_name();
    stage.a_picture_added_while_the_window_shows_takes_its_size_once_decoded();

    stage.expectations.report("images")
}
