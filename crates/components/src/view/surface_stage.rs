use morok_core::FrameStage;

/// The stages of the frame of a surface, declared in the order they run.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SurfaceStage {
    /// A resting pointer is hit-tested again over the boxes the last frame moved.
    Hover,
    /// The signals that follow the `Scroll` elements learn how far each is scrolled.
    Scrolls,
    /// The timers that fall due run what they were started for.
    Timers,
    /// The effects that wait run.
    Effects,
    /// A stage of the tree of the surface.
    Tree(FrameStage),
}
