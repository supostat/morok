/// Names a surface among the surfaces of its application: the content of the surface picks the
/// number, and the content of another surface is told it when a pointer pressed there goes
/// over this one.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct SurfaceKey(pub u64);
