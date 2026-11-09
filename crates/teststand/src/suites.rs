//! The suites of the stand, by name.

mod effects;
mod icons_images;
mod primitives;

use crate::suite::Suite;

/// The names of all suites.
pub const NAMES: [&str; 3] = [primitives::NAME, effects::NAME, icons_images::NAME];

pub fn named(name: &str) -> Option<Suite> {
    match name {
        primitives::NAME => Some(primitives::suite()),
        effects::NAME => Some(effects::suite()),
        icons_images::NAME => Some(icons_images::suite()),
        _ => None,
    }
}
