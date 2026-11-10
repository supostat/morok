use morok_core::{Point, Size};

/// How far the content of a `Scroll` element is scrolled and how large a viewport shows it,
/// as the frame the signal was last written in found them.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ScrollArea {
    pub offset: Point,
    pub viewport: Size,
}
