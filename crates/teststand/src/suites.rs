//! The suites of the stand, by name.

mod effects;
mod primitives;

use crate::suite::Suite;

/// The names of all suites.
pub const NAMES: [&str; 2] = [primitives::NAME, effects::NAME];

pub fn named(name: &str) -> Option<Suite> {
    match name {
        primitives::NAME => Some(primitives::suite()),
        effects::NAME => Some(effects::suite()),
        _ => None,
    }
}
