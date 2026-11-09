use morok_core::{ElementTree, Size};
use morok_css::SourceFile;

use crate::scene_tree::SceneTree;

/// A scripted scene: what the surface shows and what changes on it from frame to frame.
pub trait Scene {
    /// The size of the surface the scene is shown on, in CSS pixels.
    fn viewport(&self) -> Size;

    /// The stylesheet of the scene; it takes its tokens from the kit.
    fn stylesheet(&self) -> SourceFile;

    /// Fills the tree the scene is shown in. The scene keeps the IDs of the elements it changes.
    fn build(&mut self, tree: &mut SceneTree);

    /// Applies the changes that reach the tree before frame number `frame`, counted from zero,
    /// begins. A scene that stands still applies none.
    fn change(&mut self, frame: usize, tree: &mut ElementTree);
}
