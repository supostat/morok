/// Where a scroll stands in the gesture that makes it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ScrollPhase {
    /// A notch of a wheel, or a scroll of no gesture: the content moves and stops at its limits.
    Discrete,
    /// The fingers touched the trackpad and began to move. The scroll belongs to the content
    /// it began over until the gesture and its momentum end.
    GestureBegan,
    GestureChanged,
    /// The fingers left the trackpad; the system may carry the gesture on with momentum.
    GestureEnded,
    /// The system carries the gesture on after the fingers left.
    Momentum,
    MomentumEnded,
}

impl ScrollPhase {
    /// Whether the scroll is one of a gesture, which may pull the content past its limits.
    pub(crate) fn is_of_gesture(self) -> bool {
        self != Self::Discrete
    }

    /// Whether the content comes to rest after the scroll unless more of the gesture follows.
    pub(crate) fn ends_motion(self) -> bool {
        matches!(self, Self::GestureEnded | Self::MomentumEnded)
    }
}
