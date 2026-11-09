use morok_style::{BackgroundImage, Declaration, ShadowLayer, StyleSheet, Theme};

use crate::element::Element;
use crate::paint::box_frame::{BoxFrame, fitted_radii};
use crate::paint::display_item::DisplayItem;
use crate::paint::display_list::DisplayList;
use crate::paint::gradient_resolver::GradientResolver;
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
    /// background color, the background image, the inset shadows and the border.
    pub(crate) fn paint_box(self, element: &Element, display_list: &mut DisplayList) {
        let frame = Self::frame(element);
        let paint = &element.style.paint;
        self.paint_shadows(element, frame, false, display_list);
        if paint.background_color.alpha > 0.0 {
            display_list.items.push(DisplayItem::Background {
                frame,
                color: paint.background_color,
            });
        }
        if let Some(source) = paint.background_image {
            self.paint_background_image(element, source, frame, display_list);
        }
        self.paint_shadows(element, frame, true, display_list);
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
    fn paint_shadows(self, element: &Element, frame: BoxFrame, inset: bool, display_list: &mut DisplayList) {
        let Some(shadow) = element.style.paint.box_shadow else {
            return;
        };
        let layers = self.theme.resolve(shadow).layers.iter().rev();
        display_list.items.extend(
            layers
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
