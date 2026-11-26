use crate::text::shaped_glyph::ShapedGlyph;
use crate::text::shaped_line::ShapedLine;
use crate::text::shaped_run::ShapedRun;

/// The lines a [`TextShaper`](crate::TextShaper) broke a text into, with their glyphs. The
/// buffers are reused from one shaping to the next.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ShapedText {
    pub lines: Vec<ShapedLine>,
    pub runs: Vec<ShapedRun>,
    pub glyphs: Vec<ShapedGlyph>,
}

impl ShapedText {
    /// Forgets the lines and keeps the buffers for the next shaping.
    pub fn clear(&mut self) {
        self.lines.clear();
        self.runs.clear();
        self.glyphs.clear();
    }

    /// The runs of one line, each set in one font.
    pub fn runs_of(&self, line: &ShapedLine) -> &[ShapedRun] {
        &self.runs[line.runs.as_range()]
    }

    /// The glyphs of one run.
    pub fn glyphs_of(&self, run: &ShapedRun) -> &[ShapedGlyph] {
        &self.glyphs[run.glyphs.as_range()]
    }
}
