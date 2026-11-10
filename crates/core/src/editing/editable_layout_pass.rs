use morok_style::Theme;

use crate::editing::line_mode::LineMode;
use crate::element::Element;
use crate::geometry::rect::Rect;
use crate::paint::item_range::ItemRange;
use crate::text::caret_line::CaretLine;
use crate::text::caret_stop::CaretStop;
use crate::text::line_metrics::LineMetrics;
use crate::text::text_request::TextRequest;
use crate::text::text_shaper::TextShaper;
use crate::text::width_constraint::WidthConstraint;

/// Lays out the lines of the edited text of an element in the content box layout gave it: one
/// line that is as long as its text, or lines that wrap at the width of the box. White space
/// is kept as it was typed. A caret that moved since the last layout is scrolled into view.
///
/// # Panics
///
/// When the element edits no text.
pub(crate) fn lay_out(element: &mut Element, theme: &Theme, text_shaper: &mut dyn TextShaper, scale: f32) {
    let style = element.style.text;
    let box_layout = &element.layout;
    let content_box = Rect::new(
        box_layout.border.left + box_layout.padding.left,
        box_layout.border.top + box_layout.padding.top,
        (box_layout.size.width
            - box_layout.border.left
            - box_layout.padding.left
            - box_layout.padding.right
            - box_layout.border.right)
            .max(0.0),
        (box_layout.size.height
            - box_layout.border.top
            - box_layout.padding.top
            - box_layout.padding.bottom
            - box_layout.border.bottom)
            .max(0.0),
    );
    let editable = element
        .editable
        .as_deref_mut()
        .expect("only an element that edits text has its edited lines laid out");
    let width = match editable.line_mode {
        LineMode::Single => WidthConstraint::MaxContent,
        LineMode::Multiple => WidthConstraint::Definite(content_box.width),
    };
    let font = style.font_request(theme);
    let text = editable.editor.text();
    let layout = &mut editable.layout;
    text_shaper.shape_editable(
        &TextRequest {
            text,
            font,
            letter_spacing: style.letter_spacing,
            width,
        },
        &mut layout.shaped,
        &mut layout.carets,
    );
    if text.is_empty() || text.ends_with('\n') {
        let end = text.len() as u32;
        let stop = layout.carets.stops.len() as u32;
        layout.carets.stops.push(CaretStop { offset: end, x: 0.0 });
        layout.carets.lines.push(CaretLine {
            text: ItemRange { start: end, end },
            stops: ItemRange {
                start: stop,
                end: stop + 1,
            },
        });
    }
    layout.placeholder.clear();
    if text.is_empty() && !editable.placeholder.is_empty() {
        let placeholder = TextRequest {
            text: &editable.placeholder,
            font,
            letter_spacing: style.letter_spacing,
            width,
        };
        text_shaper.shape(&placeholder, None, &mut layout.placeholder);
    }
    layout.font_metrics = text_shaper.font_metrics(&font);
    let line = LineMetrics::of(&style, &layout.font_metrics, scale);
    layout.line_height = line.line_height;
    layout.baseline = line.baseline;
    layout.ascent = line.ascent;
    layout.content_height = line.content_height;
    layout.content_box = content_box;
    layout.text_align = style.text_align;
    layout.scale = scale;
    layout.text_revision = Some(editable.text_revision);
    if editable.reveals_caret {
        editable.reveal_caret();
    } else {
        editable.keep_scroll_in_range();
    }
}
