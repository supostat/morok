use morok_core::Size;

/// How far an image handed to the renderer has come.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ImageStatus {
    /// The decoder has not finished.
    Decoding,
    /// The data is not an image the system reads.
    Unreadable,
    /// The picture is ready, `Size` pixels large.
    Decoded(Size),
}
