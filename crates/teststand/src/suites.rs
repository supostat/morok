//! The suites of the stand, by name.

mod effects;
mod icons_images;
mod motion;
mod primitives;

use crate::suite::Suite;

/// The names of all suites.
pub const NAMES: [&str; 4] = [primitives::NAME, effects::NAME, icons_images::NAME, motion::NAME];

pub fn named(name: &str) -> Option<Suite> {
    match name {
        primitives::NAME => Some(primitives::suite()),
        effects::NAME => Some(effects::suite()),
        icons_images::NAME => Some(icons_images::suite()),
        motion::NAME => Some(motion::suite()),
        _ => None,
    }
}
