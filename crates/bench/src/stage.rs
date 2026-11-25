/// The stages of a frame the harness times, in the order they run.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Stage {
    /// The input or the changes of the frame reach the tree, and the stages a run declares
    /// before its layout prepare it: hover, scrolls, timers, effects, styles and pictures.
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
