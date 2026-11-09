use objc2::rc::Retained;
use objc2::runtime::ProtocolObject;
use objc2_metal::MTLTexture;

/// A texture that several threads may hold.
#[derive(Debug)]
pub(crate) struct SharedTexture(pub(crate) Retained<ProtocolObject<dyn MTLTexture>>);

// SAFETY: Metal lets any thread use a texture; writes to it are kept apart by the lock around
// the atlas that owns it.
unsafe impl Send for SharedTexture {}
