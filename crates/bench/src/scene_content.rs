use std::fmt::{self, Debug, Formatter};

use morok_core::{ElementTree, FrameScheduler, Size};
use morok_platform::{CoreTextShaper, SurfaceContent};
use morok_style::ColorScheme;

use crate::bench_error::BenchError;
use crate::scene::Scene;
use crate::scene_tree::SceneTree;

/// A scene shown in a window. The window runs the frames; the changes of the scene for the
/// next frame are applied as a frame ends, so a scene that changes asks for the frame that
/// shows the change, as a timer or an event would.
pub struct SceneContent {
    scene: Box<dyn Scene>,
    scene_tree: SceneTree,
    text_shaper: CoreTextShaper,
    frames_run: usize,
}

impl SceneContent {
    /// Builds the tree of the scene under the theme of `color_scheme`.
    pub fn new(mut scene: Box<dyn Scene>, color_scheme: ColorScheme) -> Result<Self, BenchError> {
        let scene_tree = SceneTree::of(scene.as_mut(), color_scheme)?;
        Ok(Self {
            scene,
            scene_tree,
            text_shaper: CoreTextShaper::new(),
            frames_run: 0,
        })
    }

    /// Names the window the tree asks for its frames.
    pub fn set_frame_scheduler(&mut self, frame_scheduler: Box<dyn FrameScheduler>) {
        self.scene_tree.tree_mut().set_frame_scheduler(frame_scheduler);
    }
}

impl SurfaceContent for SceneContent {
    fn frame(&mut self, viewport: Size, scale: f32) -> &ElementTree {
        let tree = self.scene_tree.tree_mut();
        tree.resolve_styles();
        tree.compute_layout(viewport, scale, &mut self.text_shaper);
        tree.build_display_list();
        self.frames_run += 1;
        self.scene.change(self.frames_run, tree);
        self.scene_tree.tree()
    }

    fn color_scheme_changed(&mut self, color_scheme: ColorScheme) {
        self.scene_tree.set_color_scheme(color_scheme);
    }
}

impl Debug for SceneContent {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("SceneContent")
            .field("frames_run", &self.frames_run)
            .finish_non_exhaustive()
    }
}
