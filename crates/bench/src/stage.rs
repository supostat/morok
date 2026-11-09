/// The stages of a frame the harness times, in the order they run.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Stage {
    /// The changes of the frame are applied to the tree and the styles they touched are resolved.
    Events,
    Layout,
    DisplayList,
    /// The display list is written into the buffers of the GPU and its draws are encoded.
    Encoding,
}

impl Stage {
    pub const ALL: [Self; 4] = [Self::Events, Self::Layout, Self::DisplayList, Self::Encoding];

    /// The name of the stage in the budgets file and the reports.
    pub fn name(self) -> &'static str {
        match self {
            Self::Events => "events",
            Self::Layout => "layout",
            Self::DisplayList => "display_list",
            Self::Encoding => "encoding",
        }
    }

    pub(crate) fn index(self) -> usize {
        self as usize
    }
}
