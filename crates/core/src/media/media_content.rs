use crate::geometry::size::Size;
use crate::media::canvas_id::CanvasId;
use crate::media::icon_id::IconId;
use crate::media::image_id::ImageId;

/// What a leaf shows besides its box: the picture of an image, an icon or a region the
/// application draws.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) enum MediaContent {
    #[default]
    None,
    Image {
        image: ImageId,
        /// The size of the picture in CSS pixels once it is decoded; none while the box stands
        /// in for it.
        natural_size: Option<Size>,
    },
    Icon(IconId),
    Canvas(CanvasId),
}
