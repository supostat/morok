use morok_style::{BackgroundImage, Declaration, ShadowLayer, StyleSheet, TextDecoration, Theme};

use crate::element::Element;
use crate::geometry::point::Point;
use crate::geometry::rect::Rect;
use crate::geometry::sides::Sides;
use crate::media::media_content::MediaContent;
use crate::media::object_placement::object_rect;
use crate::paint::box_frame::{BoxFrame, fitted_radii};
use crate::paint::display_item::DisplayItem;
use crate::paint::display_list::DisplayList;
use crate::paint::gradient_resolver::GradientResolver;
use crate::paint::item_range::ItemRange;
use crate::role::Role;
use crate::style::declaration_source::DeclarationSource;

/// Writes the display items of the box of one element.
#[derive(Clone, Copy, Debug)]
pub(crate) struct BoxPainter<'a> {
    pub(crate) style_sheet: &'a StyleSheet,
    pub(crate) theme: &'a Theme,
}

impl BoxPainter<'_> {
    pub(crate) fn frame(element: &Element) -> BoxFrame {
        BoxFrame {
            border_box: element.border_box,
            border_widths: element.style.layout.used_border_width(),
            radii: fitted_radii(element.style.paint.border_radius, &element.border_box),
        }
    }

    /// Appends what CSS paints of a box, bottom layer first: the shadows around it, the
    /// background color, the background image, the inset shadows and the border; then, for a
    /// text, its lines.
    pub(crate) fn paint_box(self, element: &Element, display_list: &mut DisplayList) {
        let frame = Self::frame(element);
        let paint = &element.style.paint;
        Self::paint_shadows(element, frame, false, display_list);
        if paint.background_color.alpha > 0.0 {
            display_list.items.push(DisplayItem::Background {
                frame,
                color: paint.background_color,
            });
        }
        if let Some(source) = paint.background_image {
            self.paint_background_image(element, source, frame, display_list);
        }
        Self::paint_shadows(element, frame, true, display_list);
        let border_is_visible = [
            (frame.border_widths.top, paint.border_color.top),
            (frame.border_widths.right, paint.border_color.right),
            (frame.border_widths.bottom, paint.border_color.bottom),
            (frame.border_widths.left, paint.border_color.left),
        ]
        .iter()
        .any(|(width, color)| *width > 0.0 && color.alpha > 0.0);
        if border_is_visible {
            display_list.items.push(DisplayItem::Border {
                frame,
                colors: paint.border_color,
            });
        }
        if element.role == Role::Text && paint.color.alpha > 0.0 {
            Self::paint_text(element, display_list);
        }
        Self::paint_media(element, &frame, display_list);
    }

    /// Appends the picture, icon or canvas of a leaf inside its content box. An image whose
    /// picture has not arrived shows its box alone.
    fn paint_media(element: &Element, frame: &BoxFrame, display_list: &mut DisplayList) {
        let content_box = Self::content_box(element, frame);
        if content_box.is_empty() {
            return;
        }
        match element.media {
            MediaContent::None => {}
            MediaContent::Image { image, natural_size } => {
                if let Some(natural) = natural_size {
                    display_list.items.push(DisplayItem::Image {
                        image,
                        dest: object_rect(natural, &content_box, element.style.paint.object_fit),
                        clip: content_box,
                    });
                }
            }
            MediaContent::Icon(icon) => display_list.items.push(DisplayItem::Icon {
                icon,
                dest: content_box,
                color: element.style.paint.color,
            }),
            MediaContent::Canvas(canvas) => display_list.items.push(DisplayItem::Canvas {
                canvas,
                dest: content_box,
            }),
        }
    }

    /// The padding box without the padding, as layout resolved it.
    fn content_box(element: &Element, frame: &BoxFrame) -> Rect {
        let padding_box = frame.padding_box();
        let padding = element
            .style
            .layout
            .padding
            .map(|side| side.against(element.border_box.width).max(0.0));
        padding_box.expanded(Sides {
            top: -padding.top,
            right: -padding.right,
            bottom: -padding.bottom,
            left: -padding.left,
        })
    }

    /// Appends the lines of a text: under each line its underline, then its glyphs run by
    /// run, then the line through it. CSS draws an underline below the glyphs and a line
    /// through them above.
    fn paint_text(element: &Element, display_list: &mut DisplayList) {
        let text_layout = &element.text_layout;
        let color = element.style.paint.color;
        let decoration = element.style.text.text_decoration;
        let metrics = &text_layout.font_metrics;
        let origin = element.border_box.origin();
        for (line_box, shaped_line) in text_layout.lines.iter().zip(&text_layout.shaped.lines) {
            let baseline = origin.y + line_box.baseline;
            let decoration_line = |top: f32| DisplayItem::TextDecoration {
                rect: Rect::new(
                    origin.x + line_box.rect.x,
                    baseline + top,
                    line_box.rect.width,
                    metrics.decoration_thickness,
                ),
                color,
            };
            if decoration == TextDecoration::Underline {
                display_list.items.push(decoration_line(metrics.underline_top));
            }
            for run in text_layout.shaped.runs_of(shaped_line) {
                let start = display_list.glyphs.len();
                display_list.glyphs.extend_from_slice(text_layout.shaped.glyphs_of(run));
                display_list.items.push(DisplayItem::GlyphRun {
                    origin: Point {
                        x: origin.x + line_box.rect.x,
                        y: baseline,
                    },
                    font: run.font,
                    glyphs: ItemRange {
                        start: start as u32,
                        end: display_list.glyphs.len() as u32,
                    },
                    color,
                });
            }
            if decoration == TextDecoration::LineThrough {
                display_list.items.push(decoration_line(metrics.line_through_top));
            }
        }
    }

    pub(crate) fn paint_outline(element: &Element, display_list: &mut DisplayList) {
        let paint = &element.style.paint;
        let width = paint.used_outline_width();
        if width > 0.0 && paint.outline_color.alpha > 0.0 {
            display_list.items.push(DisplayItem::Outline {
                frame: Self::frame(element),
                width,
                offset: paint.outline_offset,
                color: paint.outline_color,
            });
        }
    }

    /// The first layer of a shadow is the topmost in CSS, so the layers are appended last to first.
    fn paint_shadows(element: &Element, frame: BoxFrame, inset: bool, display_list: &mut DisplayList) {
        display_list.items.extend(
            element
                .lists
                .box_shadow
                .iter()
                .rev()
                .filter(|layer| layer.inset == inset && is_visible(layer))
                .map(|layer| DisplayItem::Shadow { frame, layer: *layer }),
        );
    }

    fn paint_background_image(
        self,
        element: &Element,
        source: DeclarationSource,
        frame: BoxFrame,
        display_list: &mut DisplayList,
    ) {
        let gradient = match source.declaration(self.style_sheet, &element.inline_style) {
            Declaration::BackgroundImage(BackgroundImage::Gradient(gradient)) => gradient,
            other => unreachable!("the source of background-image names {other:?}"),
        };
        let resolver = GradientResolver {
            theme: self.theme,
            current_color: element.style.paint.color,
        };
        let gradient = resolver.resolve(gradient, &frame.padding_box(), &mut display_list.gradient_stops);
        display_list.items.push(DisplayItem::Gradient { frame, gradient });
    }
}

fn is_visible(layer: &ShadowLayer) -> bool {
    layer.color.alpha > 0.0
}
