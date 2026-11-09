/// One of the four properties that hold a list of grid tracks.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum TrackList {
    TemplateColumns,
    TemplateRows,
    AutoColumns,
    AutoRows,
}

impl TrackList {
    pub(crate) const ALL: [Self; 4] = [
        Self::TemplateColumns,
        Self::TemplateRows,
        Self::AutoColumns,
        Self::AutoRows,
    ];
}
