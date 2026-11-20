use morok::style::{Color, Token};

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
