use crate::view::surface::Surface;

/// A place of the surfaces of an application: the surface that holds it now, if any, and how
/// many surfaces held it before.
#[derive(Debug)]
pub(crate) struct SurfaceSlot {
    pub(crate) generation: u32,
    pub(crate) surface: Option<Surface>,
}
