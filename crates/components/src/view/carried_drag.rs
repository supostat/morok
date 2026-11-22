use crate::view::surface_id::SurfaceId;

/// A drag of the pointer over another surface than the one whose tree began it. The session
/// stays in the tree of `source`; the tree of `over` takes the drag as one that began outside
/// it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct CarriedDrag {
    pub(crate) source: SurfaceId,
    pub(crate) over: SurfaceId,
}
