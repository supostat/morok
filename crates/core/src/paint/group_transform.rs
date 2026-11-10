use crate::element::Element;
use crate::geometry::transform::Transform;
use crate::motion::transform_step::transform_of;

/// The transform of an element in the coordinates it was laid out in: the steps of its
/// `transform` applied about its `transform-origin`.
pub(crate) fn local_transform(element: &Element) -> Transform {
    if element.lists.transform.is_empty() {
        return Transform::IDENTITY;
    }
    let border_box = &element.border_box;
    let origin_x = border_box.x + element.style.effects.transform_origin_x.against(border_box.width);
    let origin_y = border_box.y + element.style.effects.transform_origin_y.against(border_box.height);
    Transform::translation(-origin_x, -origin_y)
        .then(&transform_of(&element.lists.transform, border_box))
        .then(&Transform::translation(origin_x, origin_y))
}
