//! The suites of the stand, by name.

mod button;
mod collections;
mod controls;
mod effects;
mod icons_images;
mod motion;
mod primitives;
mod scroll;
mod text_fields;

use morok_components::STYLESHEETS;
use morok_css::SourceFile;

use crate::suite::Suite;

/// The names of all suites.
pub const NAMES: [&str; 9] = [
    primitives::NAME,
    effects::NAME,
    icons_images::NAME,
    motion::NAME,
    button::NAME,
    text_fields::NAME,
    controls::NAME,
    scroll::NAME,
    collections::NAME,
];

pub fn named(name: &str) -> Option<Suite> {
    match name {
        primitives::NAME => Some(primitives::suite()),
        effects::NAME => Some(effects::suite()),
        icons_images::NAME => Some(icons_images::suite()),
        motion::NAME => Some(motion::suite()),
        button::NAME => Some(button::suite()),
        text_fields::NAME => Some(text_fields::suite()),
        controls::NAME => Some(controls::suite()),
        scroll::NAME => Some(scroll::suite()),
        collections::NAME => Some(collections::suite()),
        _ => None,
    }
}

/// The stylesheet of a component as the kit ships it, by its path from the root of the workspace.
///
/// # Panics
///
/// When the kit ships no stylesheet at the path.
fn component_stylesheet(path: &str) -> SourceFile {
    let source = STYLESHEETS
        .iter()
        .find(|source| source.path == path)
        .unwrap_or_else(|| panic!("the kit ships no stylesheet at `{path}`"));
    SourceFile::new(source.path, source.text)
}
