/// A handle to a surface of a [`Surfaces`](crate::view::Surfaces).
///
/// Closing a surface retires its handle: the slot may be reused, but only under a new generation.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct SurfaceId {
    index: u32,
    generation: u32,
}

impl SurfaceId {
    pub(crate) const fn new(index: u32, generation: u32) -> Self {
        Self { index, generation }
    }

    pub(crate) const fn index(self) -> usize {
        self.index as usize
    }

    pub(crate) const fn generation(self) -> u32 {
        self.generation
    }

    /// A number no other surface of the application has or had: for a host that names its
    /// own windows by the surfaces they show.
    pub const fn serial(self) -> u64 {
        ((self.generation as u64) << 32) | self.index as u64
    }
}
