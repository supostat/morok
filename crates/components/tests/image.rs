mod support;

use morok_components::Image;
use morok_components::styles::CLASSES;
use morok_core::accesskit::Role as AccessibleRole;
use morok_core::{ImageId, ImageStatus, Role, Size};
use support::Stage;

const SUNSET: ImageId = ImageId(7);
const HARBOUR: ImageId = ImageId(8);
const SUNSET_SIZE: Size = Size {
    width: 120.0,
    height: 80.0,
};

#[test]
fn an_image_is_a_named_picture_of_the_image_role() {
    let mut stage = Stage::new();
    let image = stage.mount(|mount| Image::new(SUNSET, "Sunset over the bay").mount(mount));
    stage.frame();

    let tree = stage.tree();
    assert_eq!(
        (tree.role(image), tree.classes(image)),
        (Role::Image, [CLASSES.image].as_slice())
    );
    assert_eq!(tree.image(image), Some(SUNSET));
    let node = stage.accessible_node(image);
    assert_eq!(
        (node.role(), node.label()),
        (AccessibleRole::Image, Some("Sunset over the bay"))
    );
}

#[test]
fn an_image_takes_the_size_of_its_picture_from_the_host() {
    let mut stage = Stage::new();
    let image = stage.mount(|mount| Image::new(SUNSET, "Sunset").mount(mount));
    stage.frame();
    assert_eq!(
        stage.tree().border_box(image).size(),
        Size::default(),
        "while its picture decodes the image is its box alone"
    );

    stage.report_image(SUNSET, ImageStatus::Decoded(SUNSET_SIZE));
    stage.frame();

    assert_eq!(stage.tree().border_box(image).size(), SUNSET_SIZE);
}

#[test]
fn an_image_shows_the_picture_its_signal_names() {
    let mut stage = Stage::new();
    let shown = stage.runtime.signal(SUNSET);
    let image = stage.mount(move |mount| Image::new(shown, "Photo").mount(mount));
    stage.flush();
    stage.report_image(SUNSET, ImageStatus::Decoded(SUNSET_SIZE));
    stage.frame();

    stage.runtime.set(shown, SUNSET);
    stage.frame();
    assert_eq!(
        stage.tree().border_box(image).size(),
        SUNSET_SIZE,
        "the picture it shows written again keeps its size"
    );

    stage.runtime.set(shown, HARBOUR);
    stage.frame();
    assert_eq!(stage.tree().image(image), Some(HARBOUR));
    assert_eq!(
        stage.tree().border_box(image).size(),
        Size::default(),
        "another picture waits for a size of its own"
    );
}
