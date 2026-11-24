mod support;

use std::cell::Cell;
use std::rc::Rc;
use std::time::Duration;

use morok_core::{ElementId, FrameScheduler, ImageId, ImageStatus, Role, Size};
use support::Scene;

const GALLERY: &str = ".row { display: flex; align-items: flex-start; }\n\
    .column { display: block; width: 300px; }\n\
    .framed { width: 40px; height: 30px; }";

const PHOTO: ImageId = ImageId(1);
const PHOTO_SIZE: Size = Size {
    width: 64.0,
    height: 48.0,
};

/// Counts the frames a tree asked its host for.
#[derive(Debug, Default)]
struct CountingScheduler {
    requested_frames: Rc<Cell<usize>>,
}

impl FrameScheduler for CountingScheduler {
    fn request_frame(&self) {
        self.requested_frames.set(self.requested_frames.get() + 1);
    }

    fn request_frame_after(&self, _delay: Duration) {}
}

/// A row of a scene whose tree has a host, and the number of frames the tree asked for.
fn hosted_row() -> (Scene, ElementId, Rc<Cell<usize>>) {
    let mut scene = Scene::new(GALLERY);
    let row = scene.add(scene.root(), Role::Element, &["row"]);
    let requested_frames = Rc::new(Cell::new(0));
    scene.tree.set_frame_scheduler(Box::new(CountingScheduler {
        requested_frames: Rc::clone(&requested_frames),
    }));
    (scene, row, requested_frames)
}

/// Runs a frame as a host does, which tells the tree how far it has come with every picture
/// by `status`, and returns how many frames the tree asked for in it.
fn hosted_frame(scene: &mut Scene, requested_frames: &Cell<usize>, status: ImageStatus) -> usize {
    let before = requested_frames.get();
    scene.images.report(status);
    scene.frame();
    requested_frames.get() - before
}

#[test]
fn an_image_takes_its_natural_size_once_the_host_has_decoded_it() {
    let (mut scene, row, requested_frames) = hosted_row();
    let photo = scene.add(row, Role::Image, &[]);
    scene.tree.set_image(photo, PHOTO, None);

    let asked = hosted_frame(&mut scene, &requested_frames, ImageStatus::Decoding);
    assert_eq!(
        (scene.border_box(photo).size(), asked),
        (Size::default(), 1),
        "while its picture decodes the image is a box of no size of its own and the tree asks for the next frame"
    );
    assert_eq!(scene.tree.image(photo), Some(PHOTO));

    hosted_frame(&mut scene, &requested_frames, ImageStatus::Decoded(PHOTO_SIZE));
    assert_eq!(scene.border_box(photo).size(), PHOTO_SIZE);
    assert_eq!(
        hosted_frame(&mut scene, &requested_frames, ImageStatus::Decoded(PHOTO_SIZE)),
        0,
        "a tree whose pictures are sized asks for no frame"
    );
}

#[test]
fn an_unreadable_image_keeps_the_box_its_style_gives_it() {
    let (mut scene, row, requested_frames) = hosted_row();
    let framed = scene.add(row, Role::Image, &["framed"]);
    scene.tree.set_image(framed, PHOTO, None);
    hosted_frame(&mut scene, &requested_frames, ImageStatus::Decoding);
    let decoding_box = scene.border_box(framed);

    let asked = hosted_frame(&mut scene, &requested_frames, ImageStatus::Unreadable);

    assert_eq!(
        decoding_box.size(),
        Size {
            width: 40.0,
            height: 30.0
        }
    );
    assert_eq!((scene.border_box(framed), asked), (decoding_box, 0));
    assert_eq!(
        hosted_frame(&mut scene, &requested_frames, ImageStatus::Decoded(PHOTO_SIZE)),
        0,
        "the host is not asked of a picture it found unreadable again"
    );
    assert_eq!(scene.border_box(framed), decoding_box);
}

#[test]
fn an_image_outside_the_tree_asks_for_no_frame_and_is_sized_once_it_joins_it() {
    let (mut scene, row, requested_frames) = hosted_row();
    let photo = scene.tree.create(Role::Image);
    scene.tree.set_image(photo, PHOTO, None);

    assert_eq!(
        hosted_frame(&mut scene, &requested_frames, ImageStatus::Decoding),
        0,
        "no frame shows a picture outside the tree"
    );

    scene.tree.append_child(row, photo);
    hosted_frame(&mut scene, &requested_frames, ImageStatus::Decoded(PHOTO_SIZE));
    assert_eq!(scene.border_box(photo).size(), PHOTO_SIZE);
}

#[test]
fn a_removed_image_waits_for_its_size_no_longer() {
    let (mut scene, row, requested_frames) = hosted_row();
    let photo = scene.add(row, Role::Image, &[]);
    scene.tree.set_image(photo, PHOTO, None);
    hosted_frame(&mut scene, &requested_frames, ImageStatus::Decoding);

    scene.tree.remove(photo);
    let asked_before = scene.images.times_asked();
    scene.frame();

    assert_eq!(scene.images.times_asked(), asked_before);
}

#[test]
fn an_image_in_a_block_is_as_wide_as_its_picture_rather_than_the_block() {
    let (mut scene, row, requested_frames) = hosted_row();
    let column = scene.add(row, Role::Element, &["column"]);
    let photo = scene.add(column, Role::Image, &[]);
    scene.tree.set_image(photo, PHOTO, None);

    hosted_frame(&mut scene, &requested_frames, ImageStatus::Decoded(PHOTO_SIZE));

    assert_eq!(scene.border_box(column).width.to_bits(), 300.0_f32.to_bits());
    assert_eq!(scene.border_box(photo).size(), PHOTO_SIZE);
}
