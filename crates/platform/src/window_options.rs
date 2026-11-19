use morok_core::Size;

/// How a window opens.
#[derive(Clone, Debug, PartialEq)]
pub struct WindowOptions {
    pub title: String,
    /// The size of the surface inside the window frame, in CSS pixels.
    pub content_size: Size,
    /// Whether the window shows what lies behind it on the screen, blurred by the system,
    /// wherever its surface draws nothing or draws with transparency.
    pub blur_behind: bool,
}
