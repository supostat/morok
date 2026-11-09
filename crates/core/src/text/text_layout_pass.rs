use std::mem;

use morok_style::{Overflow, TextAlign, TextOverflow, Theme};

use crate::dirty::Dirty;
use crate::element_arena::ElementArena;
use crate::element_id::ElementId;
use crate::geometry::rect::Rect;
use crate::role::Role;
use crate::text::line_box::LineBox;
use crate::text::line_metrics::LineMetrics;
use crate::text::text_request::TextRequest;
use crate::text::text_shaper::TextShaper;
use crate::text::white_space_processing::processed;
use crate::text::width_constraint::WidthConstraint;

/// The end of the layout stage for text: every text element whose text, style or width changed
/// has its lines shaped in the width layout gave it and placed as CSS places line boxes.
pub(crate) struct TextLayoutPass<'a> {
    pub(crate) elements: &'a mut ElementArena,
    pub(crate) theme: &'a Theme,
    pub(crate) text_shaper: &'a mut dyn TextShaper,
    /// Device pixels to a CSS pixel on the surface.
    pub(crate) scale: f32,
    /// Scratch space for texts whose white space collapses.
    pub(crate) processed_text: &'a mut String,
}

impl TextLayoutPass<'_> {
    /// Returns whether a text was laid out, and so must be painted again.
    pub(crate) fn run(mut self) -> bool {
        if !self.elements.text_is_stale() {
            return false;
        }
        let mut laid_out = false;
        for slot in 0..self.elements.slot_count() {
            let id = self.elements.id_at(slot);
            if !self.elements.contains(id) || !self.elements[id].dirty.contains(Dirty::TEXT) {
                continue;
            }
            self.elements[id].dirty.remove(Dirty::TEXT);
            if self.shows_text(id) {
                self.lay_out(id);
                self.elements[id].dirty.insert(Dirty::PAINT);
                laid_out = true;
            }
        }
        self.elements.text_is_laid_out();
        laid_out
    }

    /// An element outside the tree or without a box is laid out when it gets one: its box
    /// changes then.
    fn shows_text(&self, id: ElementId) -> bool {
        let element = &self.elements[id];
        element.role == Role::Text && element.is_attached && element.style.layout.generates_box()
    }

    /// A line that does not fit is cut with an ellipsis when the parent asks for one and clips
    /// what overflows it sideways, as CSS requires of `text-overflow`.
    fn cuts_overflowing_lines(&self, id: ElementId) -> bool {
        self.elements[id].parent.is_some_and(|parent| {
            let style = &self.elements[parent].style;
            style.text.text_overflow == TextOverflow::Ellipsis && style.layout.overflow_x == Overflow::Hidden
        })
    }

    fn lay_out(&mut self, id: ElementId) {
        let cuts_overflowing_lines = self.cuts_overflowing_lines(id);
        let mut text_layout = mem::take(&mut self.elements[id].text_layout);
        let element = &self.elements[id];
        let style = element.style.text;
        let box_layout = &element.layout;
        let content_left = box_layout.border.left + box_layout.padding.left;
        let content_top = box_layout.border.top + box_layout.padding.top;
        let content_width =
            (box_layout.size.width - content_left - box_layout.border.right - box_layout.padding.right).max(0.0);

        let font = style.font_request(self.theme);
        let request = TextRequest {
            text: processed(&element.text, style.white_space, self.processed_text),
            font,
            letter_spacing: style.letter_spacing,
            width: if style.white_space.wraps() {
                WidthConstraint::Definite(content_width)
            } else {
                WidthConstraint::MaxContent
            },
        };
        self.text_shaper.shape(
            &request,
            cuts_overflowing_lines.then_some(content_width),
            &mut text_layout.shaped,
        );
        text_layout.font_metrics = self.text_shaper.font_metrics(&font);
        let line = LineMetrics::of(&style, &text_layout.font_metrics, self.scale);
        text_layout.lines.clear();
        for (position, shaped_line) in text_layout.shaped.lines.iter().enumerate() {
            let top = content_top + position as f32 * line.line_height;
            let left = content_left + aligned_offset(style.text_align, content_width, shaped_line.width);
            text_layout.lines.push(LineBox {
                rect: Rect::new(left, top, shaped_line.width, line.line_height),
                baseline: top + line.baseline,
            });
        }
        self.elements[id].text_layout = text_layout;
    }
}

/// How far a line starts from the left edge of the content box. The kit lays text out left to
/// right, so `start` is the left. A line wider than the box starts at its edge whatever the
/// alignment: CSS never pushes the start of a line out of reach.
fn aligned_offset(text_align: TextAlign, content_width: f32, line_width: f32) -> f32 {
    let free_space = (content_width - line_width).max(0.0);
    match text_align {
        TextAlign::Start | TextAlign::Left => 0.0,
        TextAlign::End | TextAlign::Right => free_space,
        TextAlign::Center => free_space / 2.0,
    }
}
