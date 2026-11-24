use std::fs;
use std::path::{Path, PathBuf};

use morok::components::Image;
use morok::components::view::{ElementEvent, Mount};
use morok::core::{DragEventKind, DragReach, DragStart, ElementId, ImageId, Payload, PayloadType, Propagation};
use morok::style::Class;

use crate::gallery::Gallery;
use crate::page_layout::{column, page, readout, row, section};
use crate::styles::CLASSES;

/// The fits the page shows each picture under, with the class that gives each.
const FITS: [(&str, Class); 5] = [
    ("fill", CLASSES.fit_fill),
    ("contain", CLASSES.fit_contain),
    ("cover", CLASSES.fit_cover),
    ("none", CLASSES.fit_none),
    ("scale-down", CLASSES.fit_scale_down),
];
/// The largest file the drop zone reads: a larger one is no picture worth stalling a frame on.
const LARGEST_DROPPED_FILE: u64 = 32 * 1024 * 1024;

/// A picture dropped on the zone: what the application added, and the file it was read from.
#[derive(Clone, Debug, PartialEq)]
struct DroppedPicture {
    image: ImageId,
    file: PathBuf,
}

/// The image of the kit: the two pictures of the catalog, a PNG and a JPEG, in a box of one
/// size under every `object-fit`; the PNG at its natural size beside bytes no decoder reads;
/// and a zone that shows an image file dropped on it from Finder and gives the file back when
/// its picture is dragged out of it.
pub(crate) fn mount(mount: &mut Mount<'_>, gallery: &Gallery) -> ElementId {
    let meadow_fits = fitted_pictures(mount, gallery.meadow, "Meadow");
    let dunes_fits = fitted_pictures(mount, gallery.dunes, "Dunes");
    let natural = picture_cell(mount, &[CLASSES.page_picture], gallery.meadow, "Meadow", "natural size");
    let unreadable = picture_cell(
        mount,
        &[CLASSES.page_picture, CLASSES.boxed],
        gallery.unreadable,
        "Unreadable picture",
        "unreadable bytes",
    );
    let states = row(mount, &[natural, unreadable]);
    let zone = drop_zone(mount, gallery.clone());
    let png_section = section(mount, "OBJECT-FIT · PNG", meadow_fits);
    let jpeg_section = section(mount, "OBJECT-FIT · JPEG", dunes_fits);
    let states_section = section(mount, "NATURAL SIZE AND UNREADABLE", states);
    let finder_section = section(mount, "FROM AND TO FINDER", zone);
    page(
        mount,
        "Images",
        &[png_section, jpeg_section, states_section, finder_section],
    )
}

/// The picture in a box under every fit, each with the name of its fit under it.
fn fitted_pictures(mount: &mut Mount<'_>, image: ImageId, label: &str) -> ElementId {
    let cells: Vec<ElementId> = FITS
        .iter()
        .map(|(fit_name, fit)| {
            picture_cell(
                mount,
                &[CLASSES.page_picture, CLASSES.boxed, *fit],
                image,
                label,
                fit_name,
            )
        })
        .collect();
    row(mount, &cells)
}

/// A picture in a frame of the classes, with a caption under it.
fn picture_cell(
    mount: &mut Mount<'_>,
    frame_classes: &[Class],
    image: ImageId,
    label: &str,
    caption: &str,
) -> ElementId {
    let picture = Image::new(image, label).mount(mount);
    let mut frame = mount.element();
    for class in frame_classes {
        frame.class(*class);
    }
    let frame = frame.child(picture).id();
    let caption = mount.text(caption).class(CLASSES.page_picture_caption).id();
    mount
        .element()
        .class(CLASSES.page_picture_cell)
        .child(frame)
        .child(caption)
        .id()
}

/// A zone that reads an image file dropped on it and shows its picture, and gives the file
/// back when the picture is dragged out of it; a line under it says what it holds.
fn drop_zone(mount: &mut Mount<'_>, gallery: Gallery) -> ElementId {
    let dropped = mount.runtime().signal(Vec::<DroppedPicture>::new());
    let said = mount.runtime().signal(String::from("nothing dropped yet"));
    let is_empty = mount
        .runtime()
        .memo(move |runtime| runtime.with(dropped, Vec::is_empty));
    let zone = mount
        .element()
        .class(CLASSES.page_drop_zone)
        .drop_target(&[PayloadType::Files])
        .drag_source(move |runtime| {
            let file = runtime.with(dropped, |dropped| dropped.first().map(|picture| picture.file.clone()))?;
            Some(DragStart {
                payload: Payload {
                    files: vec![file],
                    ..Payload::default()
                },
                reach: DragReach::System,
            })
        })
        .on_event(move |runtime, event| {
            if let ElementEvent::Drag(drag) = event
                && drag.kind == DragEventKind::Dropped
                && let Some(file) = drag.payload.files.first()
            {
                match read_picture(file) {
                    Ok(encoded) => {
                        if let Some(image) = gallery.add_dropped(encoded) {
                            runtime.set(
                                dropped,
                                vec![DroppedPicture {
                                    image,
                                    file: file.clone(),
                                }],
                            );
                            runtime.set(said, format!("dropped {}", file.display()));
                        }
                    }
                    Err(problem) => runtime.set(said, problem),
                }
            }
            Propagation::Continue
        })
        .show(is_empty, |mount| {
            mount
                .text("Drop an image file from Finder here")
                .class(CLASSES.page_drop_hint)
                .id()
        })
        .for_each(
            dropped,
            |picture| picture.image,
            |mount, picture| Image::new(picture.image, file_name(&picture.file)).mount(mount),
        )
        .id();
    let said = readout(mount, said);
    column(mount, &[zone, said])
}

/// The bytes of a dropped file, or what keeps the zone from reading it.
fn read_picture(file: &Path) -> Result<Vec<u8>, String> {
    let size = fs::metadata(file)
        .map_err(|error| format!("could not read {}: {error}", file.display()))?
        .len();
    if size > LARGEST_DROPPED_FILE {
        return Err(format!(
            "{} is {size} bytes, more than a picture the zone reads",
            file.display()
        ));
    }
    fs::read(file).map_err(|error| format!("could not read {}: {error}", file.display()))
}

/// The name of the file without its folders, which an assistive technology reads for the
/// picture.
fn file_name(file: &Path) -> String {
    file.file_name().map_or_else(
        || file.display().to_string(),
        |name| name.to_string_lossy().into_owned(),
    )
}
