use std::fmt::{Debug, Formatter, Result};
use std::rc::Rc;

use morok::core::ImageId;

const MEADOW_PNG: &[u8] = include_bytes!("../images/meadow.png");
const DUNES_JPEG: &[u8] = include_bytes!("../images/dunes.jpg");
/// Bytes no decoder reads: the Images page shows what an unreadable picture leaves.
const UNREADABLE: &[u8] = b"these bytes are no picture";

/// The pictures of the catalog: its own files, added to the application once, and the way a
/// page adds a picture the user drops on it. A clone is another handle to the same pictures.
#[derive(Clone)]
pub struct Gallery {
    /// A landscape PNG of 160 by 100 pixels.
    pub(crate) meadow: ImageId,
    /// A portrait JPEG of 100 by 150 pixels.
    pub(crate) dunes: ImageId,
    pub(crate) unreadable: ImageId,
    add_dropped: Rc<dyn Fn(Vec<u8>) -> Option<ImageId>>,
}

impl Gallery {
    /// Adds the pictures of the catalog through `add_image`. `add_dropped` adds a picture a
    /// page read from a file dropped on it, and gives none once the application is gone.
    pub fn new(
        mut add_image: impl FnMut(Vec<u8>) -> ImageId,
        add_dropped: impl Fn(Vec<u8>) -> Option<ImageId> + 'static,
    ) -> Self {
        Self {
            meadow: add_image(MEADOW_PNG.to_vec()),
            dunes: add_image(DUNES_JPEG.to_vec()),
            unreadable: add_image(UNREADABLE.to_vec()),
            add_dropped: Rc::new(add_dropped),
        }
    }

    /// Adds a picture read from a file dropped on a page; none once the application is gone.
    pub(crate) fn add_dropped(&self, encoded: Vec<u8>) -> Option<ImageId> {
        (self.add_dropped)(encoded)
    }
}

impl Debug for Gallery {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> Result {
        formatter
            .debug_struct("Gallery")
            .field("meadow", &self.meadow)
            .field("dunes", &self.dunes)
            .field("unreadable", &self.unreadable)
            .finish_non_exhaustive()
    }
}
