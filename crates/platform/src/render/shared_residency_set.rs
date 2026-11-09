use objc2::rc::Retained;
use objc2::runtime::ProtocolObject;
use objc2_metal::MTLResidencySet;

/// A residency set that several threads may hold.
#[derive(Debug)]
pub(crate) struct SharedResidencySet(pub(crate) Retained<ProtocolObject<dyn MTLResidencySet>>);

// SAFETY: Metal lets any thread use a residency set; changes to it are kept apart by the lock
// around the image textures that own it.
unsafe impl Send for SharedResidencySet {}
