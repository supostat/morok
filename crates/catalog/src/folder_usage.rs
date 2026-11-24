use morok::style::{Color, Token};

/// How much of its color a folder of the volume shows in a chart of the disk map.
pub(crate) const FOLDER_OPACITY: f32 = 1.0;
/// How much of the color of its folder the first folder inside it shows; every next one shows
/// less.
const FIRST_CHILD_OPACITY: f32 = 0.75;
const CHILD_OPACITY_STEP: f32 = 0.15;

/// A folder of a volume on the disk map: how much it takes, how much the folders inside it
/// take, and the token of the color it is drawn in.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FolderUsage {
    pub name: &'static str,
    pub gigabytes: f32,
    /// The folders inside it, in gigabytes, largest first.
    pub children: &'static [f32],
    pub color: Token<Color>,
}

impl FolderUsage {
    /// How much of the color of its folder the folder inside it at `position` shows in a
    /// chart of the disk map.
    pub(crate) fn child_opacity(position: usize) -> f32 {
        FIRST_CHILD_OPACITY - position as f32 * CHILD_OPACITY_STEP
    }
}
