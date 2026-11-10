use std::time::Duration;

use morok_core::{ElementTree, Size};
use morok_style::ColorScheme;

/// What a window shows. The window asks its content for a frame whenever the tree of the
/// content, or the window itself, asked the frame loop for one.
pub trait SurfaceContent {
    /// Runs the stages of a frame for a surface of `viewport` shown with `scale` device pixels
    /// to a CSS pixel at `now`, the time the frame reaches the screen on the clock of the
    /// display, and returns the tree they ran on. The window draws the display list of the tree
    /// when the frame damaged anything.
    fn frame(&mut self, viewport: Size, scale: f32, now: Duration) -> &ElementTree;

    /// The system or the application switched the appearance the surface is shown in.
    fn color_scheme_changed(&mut self, color_scheme: ColorScheme);

    /// The user turned "Reduce motion" on or off in the accessibility settings of the system.
    fn reduced_motion_changed(&mut self, reduces_motion: bool);
}
