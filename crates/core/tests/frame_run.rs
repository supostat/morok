mod support;

use std::time::Duration;

use morok_core::{ElementId, FrameOutcome, FrameStage, ImageId, ImageStatus, Role, Size, StageObserver};
use support::Scene;

const CARDS: &str = ".row { display: flex; align-items: flex-start; }\n\
    .card { width: 100px; height: 40px; background-color: var(--color-surface); }\n\
    .wide { width: 160px; }";

const PHOTO: ImageId = ImageId(3);
const PHOTO_SIZE: Size = Size {
    width: 64.0,
    height: 48.0,
};

/// Writes down the stages of a frame in the order they tell of their end.
#[derive(Debug, Default)]
struct StageLog {
    ended: Vec<FrameStage>,
}

impl StageObserver<FrameStage> for StageLog {
    fn stage_ended(&mut self, stage: FrameStage) {
        self.ended.push(stage);
    }
}

/// A row with a card in it, not framed yet.
fn row_with_a_card() -> (Scene, ElementId, ElementId) {
    let mut scene = Scene::new(CARDS);
    let row = scene.add(scene.root(), Role::Element, &["row"]);
    let card = scene.add(row, Role::Element, &["card"]);
    (scene, row, card)
}

#[test]
fn a_frame_runs_style_pictures_layout_and_the_display_list_in_that_order() {
    let (mut scene, _, _) = row_with_a_card();
    let mut log = StageLog::default();

    scene.frame_observed(scene.time, &mut log);

    assert_eq!(
        log.ended,
        [
            FrameStage::Styles,
            FrameStage::Images,
            FrameStage::Layout,
            FrameStage::DisplayList
        ]
    );
}

#[test]
fn laying_out_runs_the_stages_up_to_layout_and_leaves_the_display_list_as_the_frame_before_built_it() {
    let (mut scene, _, card) = row_with_a_card();
    scene.frame();
    let drawn = scene.tree.display_list().items().to_vec();
    scene.add_classes(card, &["wide"]);
    let mut log = StageLog::default();

    let outcome = scene.lay_out_observed(&mut log);

    assert_eq!(log.ended, [FrameStage::Styles, FrameStage::Images, FrameStage::Layout]);
    assert_eq!(
        scene.border_box(card).size(),
        Size {
            width: 160.0,
            height: 40.0
        }
    );
    assert_eq!(
        (scene.tree.display_list().items(), outcome.rebuilt_elements),
        (drawn.as_slice(), 0),
        "nothing is drawn until the display list is built"
    );
    assert!(
        scene.frame().rebuilt_elements > 0,
        "the next frame draws what the layout changed"
    );
}

#[test]
fn a_picture_the_host_has_decoded_takes_its_natural_size_in_the_same_frame() {
    let (mut scene, row, _) = row_with_a_card();
    let photo = scene.add(row, Role::Image, &[]);
    scene.tree.set_image(photo, PHOTO, None);
    scene.images.report(ImageStatus::Decoded(PHOTO_SIZE));

    scene.frame();

    assert_eq!(scene.border_box(photo).size(), PHOTO_SIZE);
    assert_eq!(scene.images.times_asked(), 1);
    scene.frame();
    assert_eq!(
        scene.images.times_asked(),
        1,
        "the host is asked only of pictures that await their size"
    );
}

#[test]
fn a_frame_counts_what_its_stages_did_and_a_frame_that_changes_nothing_counts_nothing() {
    let (mut scene, _, _) = row_with_a_card();

    let first = scene.frame();

    assert_eq!((first.styled_elements, first.laid_out_elements), (3, 3), "{first:?}");
    assert!(first.rebuilt_elements > 0, "{first:?}");
    assert_eq!(scene.frame(), FrameOutcome::default());
}

#[test]
#[should_panic(expected = "the time of a frame never goes back")]
fn a_frame_earlier_than_the_frame_before_it_panics() {
    let (mut scene, _, _) = row_with_a_card();
    scene.frame_at(Duration::from_millis(100));

    scene.frame_at(Duration::from_millis(50));
}
