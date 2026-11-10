use std::ops::Range;

use morok_style::{Color, States};

use crate::editing::editable_layout::{CARET_WIDTH, EditableLayout};
use crate::editing::editable_text::EditableText;
use crate::element::Element;
use crate::geometry::corners::Corners;
use crate::geometry::point::Point;
use crate::geometry::rect::Rect;
use crate::geometry::sides::Sides;
use crate::paint::box_frame::BoxFrame;
use crate::paint::display_item::DisplayItem;
use crate::paint::display_list::DisplayList;
use crate::paint::item_range::ItemRange;
use crate::text::shaped_text::ShapedText;
use crate::text::text_layout_pass::aligned_offset;

/// The color browsers give a placeholder that no `::placeholder` rule colors.
const PLACEHOLDER_COLOR: Color = Color {
    red: 169,
    green: 169,
    blue: 169,
    alpha: 1.0,
};

/// The highlight of selected text that no `::selection` rule gives a background.
const SELECTION_BACKGROUND: Color = Color {
    red: 30,
    green: 144,
    blue: 255,
    alpha: 1.0,
};

/// Appends what the edited text of an element shows, clipped to its content box: the
/// highlight of the selection, the text or, while it is empty, the placeholder, the underline
/// of a text being composed, and the caret. The selection and the caret are shown only while
/// the element has focus.
///
/// # Panics
///
/// When the element edits no text.
pub(crate) fn paint(element: &Element, display_list: &mut DisplayList) {
    let editable = element
        .editable
        .as_deref()
        .expect("only an element that edits text paints edited text");
    let layout = &editable.layout;
    let content_box = Rect::new(
        element.border_box.x + layout.content_box.x,
        element.border_box.y + layout.content_box.y,
        layout.content_box.width,
        layout.content_box.height,
    );
    if !editable.has_current_lines() || content_box.is_empty() {
        return;
    }
    let painter = EditablePainter {
        editable,
        layout,
        content_box,
        origin: Point {
            x: content_box.x - editable.scroll.x,
            y: content_box.y - editable.scroll.y,
        },
        text_color: element.style.paint.color,
    };
    let clip_position = display_list.items.len();
    display_list.items.push(DisplayItem::PushClip {
        frame: BoxFrame {
            border_box: content_box,
            border_widths: Sides::uniform(0.0),
            radii: Corners::default(),
        },
    });
    let has_focus = element.states.contains(States::FOCUS);
    let selection = editable.editor.selection();
    if has_focus && !selection.is_caret() {
        let background = element.style.paint.selection_background.unwrap_or(SELECTION_BACKGROUND);
        painter.paint_highlight(selection.range(), background, display_list);
    }
    if editable.editor.text().is_empty() {
        let color = element.style.paint.placeholder_color.unwrap_or(PLACEHOLDER_COLOR);
        painter.paint_lines(&layout.placeholder, color, display_list);
    } else {
        painter.paint_lines(&layout.shaped, painter.text_color, display_list);
    }
    if let Some(marked) = editable.editor.marked() {
        painter.paint_marked_underline(marked, display_list);
    }
    if has_focus && selection.is_caret() && editable.shows_caret {
        let color = element.style.text.caret_color.unwrap_or(painter.text_color);
        painter.paint_caret(selection.head, color, display_list);
    }
    if display_list.items.len() == clip_position + 1 {
        display_list.items.pop();
    } else {
        display_list.items.push(DisplayItem::PopClip);
    }
}

struct EditablePainter<'a> {
    editable: &'a EditableText,
    layout: &'a EditableLayout,
    /// The content box of the element on the surface.
    content_box: Rect,
    /// Where the top left corner of the first line lies on the surface, scrolled.
    origin: Point,
    text_color: Color,
}

impl EditablePainter<'_> {
    /// Whether any of the line box shows in the content box.
    fn shows_line(&self, line: usize) -> bool {
        let top = self.origin.y + self.layout.line_top(line);
        top < self.content_box.bottom() && top + self.layout.line_height > self.content_box.y
    }

    /// Appends the glyph runs of the lines that show. A placeholder, which is shown in place
    /// of an empty text, is laid out line for line like a text.
    fn paint_lines(&self, shaped: &ShapedText, color: Color, display_list: &mut DisplayList) {
        if color.alpha <= 0.0 {
            return;
        }
        for (line, shaped_line) in shaped.lines.iter().enumerate() {
            if !self.shows_line(line) {
                continue;
            }
            let left = aligned_offset(self.layout.text_align, self.content_box.width, shaped_line.width);
            let origin = Point {
                x: self.origin.x + left,
                y: self.origin.y + self.layout.line_top(line) + self.layout.baseline,
            };
            for run in shaped.runs_of(shaped_line) {
                let start = display_list.glyphs.len();
                display_list.glyphs.extend_from_slice(shaped.glyphs_of(run));
                display_list.items.push(DisplayItem::GlyphRun {
                    origin,
                    font: run.font,
                    glyphs: ItemRange {
                        start: start as u32,
                        end: display_list.glyphs.len() as u32,
                    },
                    color,
                });
            }
        }
    }

    /// The stretch of every line that shows which holds bytes of `range`, as a box from the
    /// top of the line `top` down and `height` high.
    fn for_each_stretch(&self, range: &Range<usize>, top: f32, height: f32, mut each: impl FnMut(Rect)) {
        let layout = self.layout;
        for line in 0..layout.line_count() {
            let carets = layout.caret_range(line);
            let (start, end) = (range.start.max(carets.start), range.end.min(carets.end));
            if start >= end || !self.shows_line(line) {
                continue;
            }
            let line_left = self.origin.x + layout.line_left(line);
            let left = line_left + layout.x_of(line, start);
            let right = line_left + layout.x_of(line, end);
            each(self.on_device_pixels(Rect::new(
                left,
                self.origin.y + layout.line_top(line) + top,
                right - left,
                height,
            )));
        }
    }

    /// The highlight of a selection covers the line box of every line it selects text in.
    fn paint_highlight(&self, range: Range<usize>, color: Color, display_list: &mut DisplayList) {
        self.for_each_stretch(&range, 0.0, self.layout.line_height, |rect| {
            display_list.items.push(DisplayItem::Fill { rect, color });
        });
    }

    /// A text being composed is underlined until the input method settles it.
    fn paint_marked_underline(&self, marked: Range<usize>, display_list: &mut DisplayList) {
        let metrics = &self.layout.font_metrics;
        let top = self.layout.baseline + metrics.underline_top;
        self.for_each_stretch(&marked, top, metrics.decoration_thickness, |rect| {
            display_list.items.push(DisplayItem::TextDecoration {
                rect,
                color: self.text_color,
            });
        });
    }

    /// A caret behind the last character of a line as wide as the box stands at the edge of
    /// the box: lines that wrap do not scroll sideways, and browsers keep the caret in view.
    fn paint_caret(&self, offset: usize, color: Color, display_list: &mut DisplayList) {
        if color.alpha <= 0.0 {
            return;
        }
        let caret = self.layout.caret_rect(offset, self.editable.affinity());
        let x = if self.editable.scrolls_sideways() {
            caret.x
        } else {
            caret.x.min((self.content_box.width - CARET_WIDTH).max(0.0))
        };
        let on_surface = self.on_device_pixels(Rect::new(
            self.origin.x + x,
            self.origin.y + caret.y,
            caret.width,
            caret.height,
        ));
        display_list.items.push(DisplayItem::Fill {
            rect: Rect::new(on_surface.x, on_surface.y, CARET_WIDTH, on_surface.height),
            color,
        });
    }

    /// The rectangle with its edges on whole device pixels, as browsers fill the boxes of a
    /// selection and of a caret.
    fn on_device_pixels(&self, rect: Rect) -> Rect {
        let scale = self.layout.scale;
        let snapped = |edge: f32| (edge * scale).round() / scale;
        let (left, top) = (snapped(rect.x), snapped(rect.y));
        Rect::new(left, top, snapped(rect.right()) - left, snapped(rect.bottom()) - top)
    }
}
