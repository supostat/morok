use crate::media::image_id::ImageId;
use crate::media::image_status::ImageStatus;

/// The port through which the tree learns how far its host has come with the pictures it
/// shows. The tree decodes nothing: the platform implements the port with its renderer.
pub trait ImageStatuses {
    /// How far the host has come with the picture `image`.
    fn image_status(&self, image: ImageId) -> ImageStatus;
}
