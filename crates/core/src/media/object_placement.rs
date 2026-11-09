use morok_style::ObjectFit;

use crate::geometry::rect::Rect;
use crate::geometry::size::Size;

/// Where the picture of `natural` size lies in `content_box` under `fit`, centered as CSS
/// places it; it may reach past the box, which then cuts it.
pub(crate) fn object_rect(natural: Size, content_box: &Rect, fit: ObjectFit) -> Rect {
    if natural.width <= 0.0 || natural.height <= 0.0 {
        return *content_box;
    }
    let contain = (content_box.width / natural.width).min(content_box.height / natural.height);
    let cover = (content_box.width / natural.width).max(content_box.height / natural.height);
    let (width, height) = match fit {
        ObjectFit::Fill => (content_box.width, content_box.height),
        ObjectFit::Contain => (natural.width * contain, natural.height * contain),
        ObjectFit::Cover => (natural.width * cover, natural.height * cover),
        ObjectFit::None => (natural.width, natural.height),
        ObjectFit::ScaleDown => {
            let scale = contain.min(1.0);
            (natural.width * scale, natural.height * scale)
        }
    };
    Rect::new(
        content_box.x + (content_box.width - width) / 2.0,
        content_box.y + (content_box.height - height) / 2.0,
        width,
        height,
    )
}

#[cfg(test)]
mod tests {
    use morok_style::ObjectFit;

    use super::object_rect;
    use crate::geometry::rect::Rect;
    use crate::geometry::size::Size;

    const BOX: Rect = Rect::new(10.0, 20.0, 120.0, 80.0);
    const WIDE: Size = Size {
        width: 64.0,
        height: 32.0,
    };

    #[test]
    fn fill_stretches_the_picture_over_the_box() {
        assert_eq!(object_rect(WIDE, &BOX, ObjectFit::Fill), BOX);
    }

    #[test]
    fn contain_fits_the_picture_inside_the_box_centered() {
        assert_eq!(
            object_rect(WIDE, &BOX, ObjectFit::Contain),
            Rect::new(10.0, 30.0, 120.0, 60.0)
        );
    }

    #[test]
    fn cover_fills_the_box_and_reaches_past_its_sides() {
        assert_eq!(
            object_rect(WIDE, &BOX, ObjectFit::Cover),
            Rect::new(-10.0, 20.0, 160.0, 80.0)
        );
    }

    #[test]
    fn none_keeps_the_size_of_the_picture() {
        assert_eq!(
            object_rect(WIDE, &BOX, ObjectFit::None),
            Rect::new(38.0, 44.0, 64.0, 32.0)
        );
    }

    #[test]
    fn scale_down_takes_the_smaller_of_none_and_contain() {
        assert_eq!(
            object_rect(WIDE, &BOX, ObjectFit::ScaleDown),
            Rect::new(38.0, 44.0, 64.0, 32.0)
        );
        let large = Size {
            width: 240.0,
            height: 60.0,
        };
        assert_eq!(
            object_rect(large, &BOX, ObjectFit::ScaleDown),
            Rect::new(10.0, 45.0, 120.0, 30.0)
        );
    }
}
