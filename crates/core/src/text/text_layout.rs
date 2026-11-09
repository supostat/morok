use crate::geometry::rect::Rect;
use crate::text::font_metrics::FontMetrics;
use crate::text::line_box::LineBox;
use crate::text::shaped_text::ShapedText;

/// The lines of a text element as they were last laid out, relative to its border box.
#[derive(Debug, Default)]
pub(crate) struct TextLayout {
    pub(crate) shaped: ShapedText,
    /// The box of every shaped line.
    pub(crate) lines: Vec<LineBox>,
    /// The metrics of the font the decorations of the lines are drawn by.
    pub(crate) font_metrics: FontMetrics,
}

impl TextLayout {
    /// The area the line boxes cover, relative to the border box; none without lines.
    pub(crate) fn bounds(&self) -> Option<Rect> {
        let (first, rest) = self.lines.split_first()?;
        Some(rest.iter().fold(first.rect, |bounds, line| bounds.union(&line.rect)))
    }
}
