use std::fmt::{self, Debug, Formatter};
use std::time::Duration;

use morok_core::accesskit::{ActionRequest, TreeUpdate};
use morok_core::{ElementTree, FrameScheduler, KeyEvent, PointerInput, ScrollInput, Size};
use morok_platform::{CoreTextShaper, SurfaceContent, SurfaceSettings};
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
    /// Builds the tree of the scene under the settings of the system.
    pub fn new(mut scene: Box<dyn Scene>, settings: SurfaceSettings) -> Result<Self, BenchError> {
        let mut scene_tree = SceneTree::of(scene.as_mut(), settings.color_scheme)?;
        scene_tree.tree_mut().set_reduced_motion(settings.reduces_motion);
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
    fn frame(&mut self, viewport: Size, scale: f32, now: Duration, show: &mut dyn FnMut(&ElementTree)) {
        let tree = self.scene_tree.tree_mut();
        tree.resolve_styles(now);
        tree.compute_layout(viewport, scale, &mut self.text_shaper);
        tree.build_display_list();
        show(tree);
        self.frames_run += 1;
        self.scene.change(self.frames_run, tree);
    }

    fn color_scheme_changed(&mut self, color_scheme: ColorScheme) {
        self.scene_tree.set_color_scheme(color_scheme);
    }

    fn reduced_motion_changed(&mut self, reduces_motion: bool) {
        self.scene_tree.tree_mut().set_reduced_motion(reduces_motion);
    }

    /// A scene is scripted: a pointer that happens to be over its window changes nothing.
    fn pointer_input(&mut self, _input: &PointerInput) {}

    /// A scene is scripted: a scroll over its window changes nothing.
    fn scroll_input(&mut self, _input: &ScrollInput) {}

    fn key_input(&mut self, _event: &KeyEvent) -> bool {
        false
    }

    fn accessibility_tree(&mut self) -> TreeUpdate {
        self.scene_tree.tree_mut().accessibility_tree()
    }

    fn accessibility_update(&mut self) -> TreeUpdate {
        self.scene_tree.tree_mut().accessibility_update()
    }

    fn accessibility_action(&mut self, _request: &ActionRequest) {}
}

impl Debug for SceneContent {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("SceneContent")
            .field("frames_run", &self.frames_run)
            .finish_non_exhaustive()
    }
}
