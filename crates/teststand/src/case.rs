use morok_core::Size;

use crate::moment::Moment;
use crate::scene::Scene;

/// One picture a suite compares: a tree on a surface of its own.
#[derive(Clone, Copy, Debug)]
pub struct Case {
    /// Lowercase kebab-case; names the directory of the etalon.
    pub name: &'static str,
    /// The size of the surface in CSS pixels.
    pub viewport: Size,
    /// Adds the elements of the case under the root of the scene.
    pub build: fn(&mut Scene),
    pub moment: Moment,
}
