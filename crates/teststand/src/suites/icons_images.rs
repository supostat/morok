use morok_components::Image;
use morok_core::{ElementId, Size};
use morok_css::SourceFile;

use crate::case::Case;
use crate::media_source::{IconSource, ImageSource};
use crate::moment::Moment;
use crate::scene::Scene;
use crate::suite::Suite;
use crate::suites::{component_stylesheet, kit_tokens};

pub(crate) const NAME: &str = "icons-images";

const STYLESHEET_PATH: &str = "crates/teststand/suites/icons-images.css";
const STYLESHEET: &str = include_str!("../../suites/icons-images.css");

/// The icons of the kit are its SVG files; a stand case draws the same file on both sides.
const FOLDER: IconSource = IconSource {
    svg: include_str!("../../../../design/icons/folder.svg"),
};
const GEAR: IconSource = IconSource {
    svg: include_str!("../../../../design/icons/gear.svg"),
};
const SEARCH: IconSource = IconSource {
    svg: include_str!("../../../../design/icons/search.svg"),
};
const STAR: IconSource = IconSource {
    svg: include_str!("../../../../design/icons/star.svg"),
};
const CLOUD: IconSource = IconSource {
    svg: include_str!("../../../../design/icons/cloud.svg"),
};
const TRASH: IconSource = IconSource {
    svg: include_str!("../../../../design/icons/trash.svg"),
};
const FLAG: IconSource = IconSource {
    svg: include_str!("../../../../design/icons/flag.svg"),
};

const SKY_PNG: ImageSource = ImageSource {
    encoded: include_bytes!("../../suites/images/sky.png"),
    media_type: Some("image/png"),
};
const SKY_JPEG: ImageSource = ImageSource {
    encoded: include_bytes!("../../suites/images/sky.jpg"),
    media_type: Some("image/jpeg"),
};

/// Bytes no decoder reads.
const UNREADABLE: ImageSource = ImageSource {
    encoded: b"not an image",
    media_type: None,
};

const OBJECT_FITS: [&str; 5] = ["fill", "contain", "cover", "none", "scale-down"];

/// Icons at every size of the design book, pictures under every `object-fit`, and the image
/// of the kit.
pub(crate) fn suite() -> Suite {
    Suite {
        name: NAME,
        tokens: kit_tokens(),
        component_stylesheets: vec![component_stylesheet("crates/components/src/image.css")],
        stylesheets: vec![SourceFile::new(STYLESHEET_PATH, STYLESHEET)],
        cases: vec![
            case("icons", 420.0, 200.0, icons),
            case("images", 580.0, 260.0, images),
            case("image-component", 520.0, 114.0, image_component),
        ],
    }
}

fn case(name: &'static str, width: f32, height: f32, build: fn(&mut Scene)) -> Case {
    Case {
        name,
        viewport: Size { width, height },
        build,
        moment: Moment::AtRest,
    }
}

fn stage(scene: &mut Scene) -> ElementId {
    scene.add(scene.root(), &["stage"])
}

fn icons(scene: &mut Scene) {
    let stage = stage(scene);
    let sizes = scene.add(stage, &["row"]);
    scene.add_icon(sizes, &["icon", "tiny"], FOLDER);
    scene.add_icon(sizes, &["icon", "small"], FOLDER);
    scene.add_icon(sizes, &["icon", "compact"], FOLDER);
    scene.add_icon(sizes, &["icon"], FOLDER);
    scene.add_icon(sizes, &["icon", "large"], FOLDER);
    scene.add_icon(sizes, &["icon", "huge"], FOLDER);
    let glyphs = scene.add(stage, &["row", "muted"]);
    for icon in [GEAR, SEARCH, STAR, CLOUD, TRASH] {
        scene.add_icon(glyphs, &["icon"], icon);
    }
    let colored = scene.add(stage, &["row", "accented"]);
    scene.add_icon(colored, &["icon", "large"], STAR);
    scene.add_icon(colored, &["icon", "large"], FLAG);
    scene.add_icon(colored, &["icon"], FLAG);
}

fn images(scene: &mut Scene) {
    let stage = stage(scene);
    let pictures = scene.add(stage, &["pictures"]);
    for fit in OBJECT_FITS {
        scene.add_image(pictures, &["picture", fit], SKY_PNG);
    }
    for fit in OBJECT_FITS {
        scene.add_image(pictures, &["picture", "tall", fit], SKY_JPEG);
    }
}

/// The image of the kit: a picture at its natural size, the same picture in a box of the suite
/// under `cover` and under `contain`, and an unreadable picture, which leaves the box alone.
fn image_component(scene: &mut Scene) {
    let stage = stage(scene);
    let frames = scene.add(stage, &["image-frames"]);
    let sky = scene.picture(SKY_PNG);
    let unreadable = scene.picture(UNREADABLE);
    let natural = scene.add(frames, &["image-frame"]);
    scene.mount(natural, move |mount| Image::new(sky, "Sky").mount(mount));
    for fit in ["cover", "contain"] {
        let frame = scene.add(frames, &["image-frame", "boxed", fit]);
        scene.mount(frame, move |mount| Image::new(sky, "Sky").mount(mount));
    }
    let frame = scene.add(frames, &["image-frame", "boxed"]);
    scene.mount(frame, move |mount| Image::new(unreadable, "Unreadable").mount(mount));
}
