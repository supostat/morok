use crate::geometry::size::Size;

/// How far the host has come with an image an application handed it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ImageStatus {
    /// The decoder has not finished.
    Decoding,
    /// The data is not an image the system reads.
    Unreadable,
    /// The picture is ready, `Size` pixels large.
    Decoded(Size),
}
