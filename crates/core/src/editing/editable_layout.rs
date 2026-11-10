use std::ops::Range;

use morok_style::TextAlign;

use crate::editing::caret_affinity::CaretAffinity;
use crate::geometry::rect::Rect;
use crate::text::caret_line::CaretLine;
use crate::text::caret_lines::CaretLines;
use crate::text::caret_stop::CaretStop;
use crate::text::font_metrics::FontMetrics;
use crate::text::shaped_text::ShapedText;
use crate::text::text_layout_pass::aligned_offset;

/// How wide a caret is, in CSS pixels: the width browsers draw it with.
pub(crate) const CARET_WIDTH: f32 = 1.0;

/// The lines of an edited text as they were last laid out. Distances are measured from the top
/// left corner of the content box, before the lines are scrolled.
#[derive(Debug)]
pub(crate) struct EditableLayout {
    pub(crate) shaped: ShapedText,
    /// The lines as a caret moves through them: those of `shaped`, and after them the empty
    /// line a caret stands in behind the last line break or in a text without characters.
    pub(crate) carets: CaretLines,
    /// The lines of the placeholder, set only while the text is empty.
    pub(crate) placeholder: ShapedText,
    pub(crate) font_metrics: FontMetrics,
    pub(crate) line_height: f32,
    /// From the top of a line box down to its baseline.
    pub(crate) baseline: f32,
    /// From the baseline up to the top of the content area of the font.
    pub(crate) ascent: f32,
    /// The height of the content area of the font: how high a caret is.
    pub(crate) content_height: f32,
    /// The content box of the element, relative to its border box.
    pub(crate) content_box: Rect,
    pub(crate) text_align: TextAlign,
    /// Device pixels to a CSS pixel on the surface the lines were laid out for.
    pub(crate) scale: f32,
    /// The revision of the text the lines were laid out for; none before the first layout.
    pub(crate) text_revision: Option<u64>,
}

impl Default for EditableLayout {
    /// The layout of a text that was never laid out: no lines, in a box of no size.
    fn default() -> Self {
        Self {
            shaped: ShapedText::default(),
            carets: CaretLines::default(),
            placeholder: ShapedText::default(),
            font_metrics: FontMetrics::default(),
            line_height: 0.0,
            baseline: 0.0,
            ascent: 0.0,
            content_height: 0.0,
            content_box: Rect::ZERO,
            text_align: TextAlign::Start,
            scale: 1.0,
            text_revision: None,
        }
    }
}

impl EditableLayout {
    pub(crate) fn line_count(&self) -> usize {
        self.carets.lines.len()
    }

    fn stops(&self, line: usize) -> &[CaretStop] {
        self.carets.stops_of(&self.carets.lines[line])
    }

    /// The places a caret may stand at in the line, from its first to its last.
    pub(crate) fn caret_range(&self, line: usize) -> Range<usize> {
        let stops = self.stops(line);
        let (first, last) = (stops[0], stops[stops.len() - 1]);
        first.offset as usize..last.offset as usize
    }

    /// Whether the line ends where the next one starts: it wraps, with no line break between.
    pub(crate) fn wraps_into_next(&self, line: usize) -> bool {
        line + 1 < self.line_count() && self.caret_range(line).end == self.caret_range(line + 1).start
    }

    /// The line a caret at `offset` stands in.
    pub(crate) fn line_of(&self, offset: usize, affinity: CaretAffinity) -> usize {
        let lines_before = self
            .carets
            .lines
            .partition_point(|line: &CaretLine| line.text.start as usize <= offset);
        let line = lines_before.saturating_sub(1);
        let stands_at_a_wrap = line > 0 && self.caret_range(line).start == offset && self.wraps_into_next(line - 1);
        if affinity == CaretAffinity::Upstream && stands_at_a_wrap {
            line - 1
        } else {
            line
        }
    }

    /// How far from the start of the line a caret at `offset` stands. An offset between two
    /// stops of the line stands at the stop before it.
    pub(crate) fn x_of(&self, line: usize, offset: usize) -> f32 {
        let stops = self.stops(line);
        let stops_before = stops.partition_point(|stop| stop.offset as usize <= offset);
        stops[stops_before.saturating_sub(1)].x
    }

    /// The place in the line nearest to a point `x` from the start of the line.
    pub(crate) fn offset_at(&self, line: usize, x: f32) -> usize {
        let nearest = self
            .stops(line)
            .iter()
            .min_by(|first, second| (first.x - x).abs().total_cmp(&(second.x - x).abs()))
            .expect("a line has a stop at its start");
        nearest.offset as usize
    }

    /// The line a point `y` below the top of the first line lies in; the first line above
    /// the text and the last below it.
    pub(crate) fn line_at(&self, y: f32) -> usize {
        let line = (y / self.line_height).floor().max(0.0) as usize;
        line.min(self.line_count() - 1)
    }

    pub(crate) fn line_top(&self, line: usize) -> f32 {
        line as f32 * self.line_height
    }

    /// How far the line starts from the left of the content box, by `text-align`.
    pub(crate) fn line_left(&self, line: usize) -> f32 {
        let width = self.shaped.lines.get(line).map_or(0.0, |shaped| shaped.width);
        aligned_offset(self.text_align, self.content_box.width, width)
    }

    /// The box of a caret at `offset`: as wide as a caret and as high as the content area of
    /// the font, on the baseline of its line.
    pub(crate) fn caret_rect(&self, offset: usize, affinity: CaretAffinity) -> Rect {
        let line = self.line_of(offset, affinity);
        Rect::new(
            self.line_left(line) + self.x_of(line, offset),
            self.line_top(line) + self.baseline - self.ascent,
            CARET_WIDTH,
            self.content_height,
        )
    }

    /// How wide the text is: up to the last place a caret may stand at in its widest line.
    pub(crate) fn text_width(&self) -> f32 {
        (0..self.line_count())
            .map(|line| {
                let stops = self.stops(line);
                self.line_left(line) + stops[stops.len() - 1].x
            })
            .fold(0.0, f32::max)
    }

    pub(crate) fn text_height(&self) -> f32 {
        self.line_count() as f32 * self.line_height
    }

    /// How many lines the content box shows whole; one when it shows less.
    pub(crate) fn lines_in_view(&self) -> usize {
        ((self.content_box.height / self.line_height).floor() as usize).max(1)
    }
}
