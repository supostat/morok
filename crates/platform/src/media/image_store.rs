use std::collections::HashMap;
use std::sync::{Arc, Mutex, PoisonError};
use std::thread;

use morok_core::ImageId;

use crate::media::decoded_image::DecodedImage;
use crate::media::image_decoder::decode;

/// What the store knows of one image.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum ImageState {
    /// The decoder has not finished.
    Decoding,
    /// The data is not an image the system reads.
    Unreadable,
    Decoded(Arc<DecodedImage>),
}

/// The images of a renderer: handed in encoded, decoded on a thread of their own, kept decoded
/// for the GPU to take. Any thread may ask about an image.
#[derive(Debug, Default)]
pub(crate) struct ImageStore {
    images: Arc<Mutex<HashMap<ImageId, ImageState>>>,
    next_id: u32,
}

impl ImageStore {
    /// Takes the encoded bytes and starts decoding them; the image is `Decoding` until then.
    pub(crate) fn add(&mut self, encoded: Vec<u8>) -> ImageId {
        let id = ImageId(self.next_id);
        self.next_id += 1;
        self.lock().insert(id, ImageState::Decoding);
        let images = Arc::clone(&self.images);
        thread::Builder::new()
            .name(format!("morok image {}", id.0))
            .spawn(move || {
                let state =
                    decode(&encoded).map_or(ImageState::Unreadable, |decoded| ImageState::Decoded(Arc::new(decoded)));
                images.lock().unwrap_or_else(PoisonError::into_inner).insert(id, state);
            })
            .expect("a thread decodes the image");
        id
    }

    /// The state of the image, or none for an id the store never gave out.
    pub(crate) fn state(&self, image: ImageId) -> Option<ImageState> {
        self.lock().get(&image).cloned()
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, HashMap<ImageId, ImageState>> {
        self.images.lock().unwrap_or_else(PoisonError::into_inner)
    }
}
