/// Which length of a grid track size is meant.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum TrackBound {
    Minimum,
    Maximum,
    /// The limit of a `fit-content()` track.
    FitContentLimit,
}

impl TrackBound {
    pub(crate) const ALL: [Self; 3] = [Self::Minimum, Self::Maximum, Self::FitContentLimit];
}
