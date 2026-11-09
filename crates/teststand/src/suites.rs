//! The suites of the stand, by name.

mod primitives;

use crate::suite::Suite;

/// The names of all suites.
pub const NAMES: [&str; 1] = [primitives::NAME];

pub fn named(name: &str) -> Option<Suite> {
    match name {
        primitives::NAME => Some(primitives::suite()),
        _ => None,
    }
}
