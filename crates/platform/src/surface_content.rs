use morok_core::{ElementTree, Size};
use morok_style::ColorScheme;

/// What a window shows. The window asks its content for a frame whenever the tree of the
/// content, or the window itself, asked the frame loop for one.
pub trait SurfaceContent {
    /// Runs the stages of a frame for a surface of `viewport` shown with `scale` device pixels
    /// to a CSS pixel, and returns the tree they ran on. The window draws the display list of
    /// the tree when the frame damaged anything.
    fn frame(&mut self, viewport: Size, scale: f32) -> &ElementTree;

    /// The system or the application switched the appearance the surface is shown in.
    fn color_scheme_changed(&mut self, color_scheme: ColorScheme);
}
