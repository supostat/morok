use morok_core::Size;

/// How a window opens.
#[derive(Clone, Debug, PartialEq)]
pub struct WindowOptions {
    pub title: String,
    /// The size of the surface inside the window frame, in CSS pixels.
    pub content_size: Size,
}
