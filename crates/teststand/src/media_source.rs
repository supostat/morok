/// An icon as both sides of the stand take it: the text of its SVG file.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct IconSource {
    pub svg: &'static str,
}

/// An image as both sides of the stand take it: the encoded file and its media type.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ImageSource {
    pub encoded: &'static [u8],
    /// `image/png` or `image/jpeg`: what the data URL of the page names.
    pub media_type: &'static str,
}
