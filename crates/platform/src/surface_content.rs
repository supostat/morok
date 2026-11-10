use std::time::Duration;

use morok_core::accesskit::{ActionRequest, TreeUpdate};
use morok_core::{ElementTree, KeyEvent, PointerInput, Size};
use morok_style::ColorScheme;

/// What a window shows. The window asks its content for a frame whenever the tree of the
/// content, or the window itself, asked the frame loop for one.
pub trait SurfaceContent {
    /// Runs the stages of a frame for a surface of `viewport` shown with `scale` device pixels
    /// to a CSS pixel at `now`, the time the frame reaches the screen on the clock of the
    /// display, and hands the tree they ran on to `show`, once. The window draws the display
    /// list of the tree when the frame damaged anything.
    fn frame(&mut self, viewport: Size, scale: f32, now: Duration, show: &mut dyn FnMut(&ElementTree));

    /// The system or the application switched the appearance the surface is shown in.
    fn color_scheme_changed(&mut self, color_scheme: ColorScheme);

    /// The user turned "Reduce motion" on or off in the accessibility settings of the system.
    fn reduced_motion_changed(&mut self, reduces_motion: bool);

    /// The pointer moved over the surface, pressed or released a button on it, or left it.
    fn pointer_input(&mut self, input: &PointerInput);

    /// A key went down or up while the surface had the keyboard. Returns whether the content
    /// used the key; a key it did not use goes on to the window and the application.
    fn key_input(&mut self, event: &KeyEvent) -> bool;

    /// The whole accessibility tree of the surface: asked for once, when an assistive
    /// technology starts to read the surface.
    fn accessibility_tree(&mut self) -> TreeUpdate;

    /// What the last frame changed of the accessibility tree: asked for after every frame
    /// for as long as an assistive technology reads the surface.
    fn accessibility_update(&mut self) -> TreeUpdate;

    /// An assistive technology asks a node of the accessibility tree for an action.
    fn accessibility_action(&mut self, request: &ActionRequest);
}
