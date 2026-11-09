use objc2::runtime::ProtocolObject;
use objc2_metal::{MTLDrawable, MTLResidencySet};

/// The drawable of a layer that a frame is shown through.
#[derive(Clone, Copy, Debug)]
pub(crate) struct ScreenTarget<'a> {
    pub(crate) drawable: &'a ProtocolObject<dyn MTLDrawable>,
    /// The residency set of the layer: it holds the texture of the drawable.
    pub(crate) residency_set: &'a ProtocolObject<dyn MTLResidencySet>,
}
